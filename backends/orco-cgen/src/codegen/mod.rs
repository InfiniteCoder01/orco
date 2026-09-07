use orco::ir::Instr;
use std::collections::HashMap;

// mod control_flow;
// mod intrinsics;
// mod value;
// use value::ValueInfo;

/// Context for code generation.
pub struct Context<'a> {
    /// Reference to a module to reference other symbols.
    pub module: &'a orco::Module,
    /// Body to get the code from.
    pub body: &'a orco::Body,
    /// Map for instantiation generics.
    type_params: &'a HashMap<orco::Symbol, orco::Type>,
    /// Actual disambiguated C-compatible variable names.
    var_names: Vec<String>,
}

/// Makes all names unique by appending suffixes.
fn disambiguate(names: &mut Vec<String>) {
    let mut idx = HashMap::<String, Option<u32>>::new();
    for name in &mut *names {
        if let Some(count) = idx.get_mut(name) {
            *count = Some(0);
        } else {
            idx.insert(name.clone(), None);
        }
    }

    use std::fmt::Write;
    for name in names {
        if let Some(idx) = idx.get_mut(name).and_then(Option::as_mut) {
            write!(name, "{idx}").unwrap();
            *idx += 1;
        }
    }
}

impl<'a> Context<'a> {
    #[allow(missing_docs)]
    pub fn new(
        module: &'a orco::Module,
        body: &'a orco::Body,
        type_params: &'a HashMap<orco::Symbol, orco::Type>,
    ) -> Self {
        let mut var_names = Vec::with_capacity(body.variables.len());
        for (idx, var) in body.variables.iter().enumerate() {
            var_names.push(match &var.name {
                Some(name) => crate::cname(name.into()),
                None if var.arg => format!("arg{idx}"),
                None => "var".to_owned(),
            });
        }

        disambiguate(&mut var_names);

        Self {
            module,
            body,
            var_names,
            type_params,
        }
    }

    /// Codegen an instruction at `idx` with it's arguments into `f`.
    pub fn instr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        mut idx: usize,
    ) -> Result<usize, std::fmt::Error> {
        match self.body.instructions[idx] {
            Instr::IConst(value, _) => write!(f, "{value}ll").map(|_| idx + 1),
            Instr::UConst(value, _) => write!(f, "{value}ull").map(|_| idx + 1),
            Instr::FConst(value, _) => write!(f, "{value}l").map(|_| idx + 1),
            Instr::BConst(value) => write!(f, "{value}").map(|_| idx + 1),

            Instr::Global(id) => {
                let symbol = self.body.symbol(id);
                let mut generics = symbol.generics.clone();
                for ty in &mut generics {
                    ty.instantiate(self.type_params);
                }
                let name = self.module.monomorphized_name(symbol.name, &generics);
                write!(f, "{}", crate::cname(name)).map(|_| idx + 1)
            }
            Instr::Var(id) => write!(f, "{}", self.var_names[id.0 as usize]).map(|_| idx + 1),
            Instr::Field(field_idx) => {
                let ty = self.module.inline_ty(self.body.value_ty(idx + 1));
                idx = self.instr(f, idx + 1)?;
                let orco::Type::Struct { fields } = ty else {
                    panic!("trying to access field #{field_idx} on a non-struct type {ty}");
                };

                match &fields[field_idx as usize].0 {
                    Some(name) => write!(f, ".{name}")?,
                    None => write!(f, "._{field_idx}")?,
                }

                Ok(idx)
            }
            Instr::Assign => {
                idx = self.instr(f, idx + 1)?;
                write!(f, " = ")?;
                self.instr(f, idx)
            }

            Instr::AcfLabel(label) => {
                todo!()
                //     write!(f, "{}:", self.label_debug_name(label)).map(|_| idx + 1)
            }
            Instr::AcfJump(label) => {
                todo!()
                //     write!(f, "jump {}", self.label_debug_name(label)).map(|_| idx + 1)
            }
            Instr::AcfCJump(label) => {
                todo!()
                //     write!(f, "if ")?;
                //     idx = self.instr(module, f, idx + 1)?;
                //     write!(f, " jump {}", self.label_debug_name(label))?;
                //     Ok(idx)
            }

            Instr::Call(args) => {
                idx = self.instr(f, idx + 1)?;
                write!(f, "(")?;
                for i in 0..args {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    idx = self.instr(f, idx)?;
                }
                write!(f, ")").map(|_| idx)
            }

            Instr::Return(has_value) => {
                write!(f, "return")?;
                if has_value {
                    write!(f, " ")?;
                    self.instr(f, idx + 1)
                } else {
                    Ok(idx + 1)
                }
            }

            Instr::Intrinsic(orco::ir::Intrinsic::AggregateInt(count)) => {
                let mut value = 0;
                for i in 0..count {
                    match self.body.instructions[idx + i as usize] {
                        Instr::IConst(_, integer_size) => todo!(),
                        Instr::UConst(_, integer_size) => todo!(),
                        instr => panic!("int aggregation applied to invalid instruction {instr}"),
                    }
                }
                Ok(idx + 1 + count as usize)
            }

            Instr::Intrinsic(intr) if intr.infix() => {
                idx += 1;
                write!(f, "(")?;
                for i in 0..intr.arg_count() {
                    if i > 0 {
                        write!(f, " {intr} ")?;
                    }
                    idx = self.instr(f, idx)?;
                }
                write!(f, ")")?;
                Ok(idx)
            }

            Instr::Intrinsic(intr) => {
                write!(f, "{intr}")?;
                idx += 1;
                for _ in 0..intr.arg_count() {
                    write!(f, " ")?;
                    idx = self.instr(f, idx)?;
                }
                Ok(idx)
            }

            instr => {
                // todo!("{instr}");
                write!(f, "{instr}")?;
                idx += 1;
                // let args = instr.arg_count();
                // if args > 0 {
                //     idx = debug_args(idx, f, args)?;
                // }
                Ok(idx)
            }
        }
    }
}

impl std::fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.body.variables.is_empty() && self.body.instructions.is_empty() {
            return write!(f, "{{}}");
        }

        writeln!(f, "{{")?;
        for (idx, var) in self.body.variables.iter().enumerate() {
            if var.arg {
                continue;
            }

            writeln!(
                f,
                "  {};",
                crate::types::FmtType {
                    ty: &var.ty.copy_instantiate(self.type_params),
                    constant: false,
                    name: Some(&self.var_names[idx])
                }
            )?;
        }

        let mut idx = 0;
        while idx < self.body.instructions.len() {
            // if matches!(self.body.instructions[idx], Instr::AcfLabel(..)) {
            //     idx = body.debug_instr(module, f, idx + 1)?;
            //     writeln!(f)?;
            //     continue;
            // }

            write!(f, "  ")?;
            idx = self.instr(f, idx)?;
            writeln!(f, ";")?;
        }

        write!(f, "}}")
    }
}

// impl oc::BodyCodegen for Codegen<'_> {
//     fn comment(&mut self, comment: &str) {
//         for line in comment.split('\n') {
//             self.line(format_args!("// {line}"));
//         }
//     }

//     fn type_of(&self, id: usize) -> orco::Type {
//         self.values[&id].ty.clone()
//     }

//     fn declare_var(&mut self, mut ty: orco::Type, name: Option<&str>) -> oc::Variable {
//         self.backend.intern_type(&mut ty, false);
//         let id = self.variables.len();
//         let mut name = name.map_or_else(|| format!("var{id}"), |name| crate::symname(name.into())); // TODO: Not ideal
//         if self.variable_names.contains(&name) {
//             for disambiguator in 1.. {
//                 let disambiguated = format!("{name}{disambiguator}");
//                 if !self.variable_names.contains(&disambiguated) {
//                     name = disambiguated;
//                     break;
//                 }
//             }
//         }
//         self.variable_names.insert(name.clone());

//         if !matches!(&ty, orco::Type::Struct { fields } if fields.is_empty()) {
//             self.line(format_args!(
//                 "{};",
//                 crate::types::FmtType {
//                     ty: &ty,
//                     constant: false,
//                     name: Some(&name),
//                 }
//             ));
//         }

//         self.variables.push(VariableInfo { name, ty });
//         oc::Variable(id)
//     }

//     fn assign(&mut self, target: oc::Place, value: oc::Value) {
//         let target = self.place(target).expression;
//         let value = self.use_value(value).expression;
//         self.line(format_args!("{target} = {value};"));
//     }

//     fn iconst(&mut self, value: i128, size: orco::types::IntegerSize) -> oc::Value {
//         self.mk_value(ValueInfo::new(value.to_string(), orco::Type::Integer(size))) // TODO: Literal sizes
//     }

//     fn uconst(&mut self, value: u128, size: orco::types::IntegerSize) -> oc::Value {
//         self.mk_value(ValueInfo::new(
//             value.to_string(),
//             orco::Type::Unsigned(size),
//         )) // TODO: Literal sizes
//     }

//     fn fconst(&mut self, value: f64, size: u16) -> oc::Value {
//         self.mk_value(ValueInfo::new(value.to_string(), orco::Type::Float(size))) // TODO: Literal sizes
//     }

//     fn bconst(&mut self, value: bool) -> oc::Value {
//         self.mk_value(ValueInfo::new(value.to_string(), orco::Type::Bool))
//     }

//     fn read(&mut self, place: oc::Place) -> oc::Value {
//         let place = self.place(place);
//         self.mk_value(place)
//     }

//     fn reference(&mut self, place: oc::Place, mutable: bool) -> oc::Value {
//         let mut place = self.place(place);
//         place.expression.insert(0, '&');
//         place.ty = orco::Type::Ptr(Box::new(place.ty), mutable);
//         self.mk_value(place)
//     }

//     fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
//         let func = self.use_value(func);
//         let ty = match func.ty {
//             orco::Type::FnPtr {
//                 params,
//                 return_type,
//             } => {
//                 assert_eq!(params.len(), args.len());
//                 return_type
//             }
//             ty => panic!("trying to call {ty:#?} (which is not a function)"),
//         };

//         let mut call = func.expression;
//         call.push('(');
//         for (idx, arg) in args.into_iter().enumerate() {
//             let arg = self.use_value(arg);
//             if idx > 0 {
//                 call.push_str(", ");
//             }
//             call.push_str(&arg.expression);
//         }
//         call.push(')');

//         match ty {
//             Some(rt) => Some(self.mk_value(ValueInfo::new(call, *rt))),
//             None => {
//                 self.line(format_args!("{call};"));
//                 None
//             }
//         }
//     }

//     fn return_(&mut self, value: Option<oc::Value>) {
//         if let Some(value) = value {
//             let value = self.use_value(value).expression;
//             self.line(format_args!("return {value};"));
//         } else {
//             self.line(format_args!("return;"));
//         }
//     }

//     fn intrinsics(&mut self) -> impl oc::Intrinsics + '_ {
//         self
//     }

//     fn acf(&mut self) -> impl oc::AcfCodegen + '_ {
//         self
//     }

//     fn bcf(&mut self) -> impl oc::BcfCodegen + '_ {
//         self
//     }
// }

// impl std::ops::Drop for Codegen<'_> {
//     fn drop(&mut self) {
//         self.body.push('}');
//         self.backend.define(std::mem::take(&mut self.body));
//     }
// }
