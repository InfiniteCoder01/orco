use orco::ir::Instr;
use std::collections::HashMap;

// mod control_flow;
mod intrinsics;
// mod value;
// use value::ValueInfo;

/// Context for code generation.
pub struct Context<'a> {
    /// Reference to a module to reference other symbols.
    pub module: &'a orco::Module,
    /// Body to get the code from.
    pub body: &'a orco::Body,
    /// Actual disambiguated C-compatible variable names.
    var_names: Vec<String>,
    /// Actual disambiguated C-compatible label names.
    label_names: Vec<String>,
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
    pub fn new(module: &'a orco::Module, body: &'a orco::Body) -> Self {
        let mut var_names = Vec::with_capacity(body.variables.len());
        for (idx, var) in body.variables.iter().enumerate() {
            var_names.push(match &var.name {
                Some(name) => crate::cname(name.into()),
                None if var.arg => format!("arg{idx}"),
                None => "var".to_owned(),
            });
        }

        disambiguate(&mut var_names);

        let mut label_names = Vec::with_capacity(body.label_names.len());
        for label in body.label_names.iter() {
            label_names.push(match &label {
                Some(name) => crate::cname(name.into()),
                None => "label".to_owned(),
            });
        }

        disambiguate(&mut label_names);

        Self {
            module,
            body,
            var_names,
            label_names,
        }
    }

    /// Codegen an instruction at `idx` with it's arguments into `f`.
    /// Set precedence to 255 to allow all expressions unparenthesised.
    /// See https://en.cppreference.com/cpp/language/operator_precedence for other values.
    pub fn instr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        mut idx: usize,
        precedence: u8,
    ) -> Result<usize, std::fmt::Error> {
        use orco::types::IntegerSize;
        fn int_size_suffix(size: IntegerSize) -> &'static str {
            match size {
                IntegerSize::Bits(bits) if bits > 32 => "ll",
                IntegerSize::Bits(bits) if bits > 16 => "l",
                IntegerSize::Bits(_) => "",
                IntegerSize::Size => "ll",
            }
        }

        match self.body.instructions[idx] {
            Instr::IConst(value, size) => {
                write!(f, "{value}{}", int_size_suffix(size)).map(|_| idx + 1)
            }
            Instr::UConst(value, size) => {
                write!(f, "{value}u{}", int_size_suffix(size)).map(|_| idx + 1)
            }
            Instr::FConst(value, _) => write!(f, "{value}").map(|_| idx + 1),
            Instr::BConst(value) => write!(f, "{value}").map(|_| idx + 1),

            Instr::Global(id) => {
                let symbol = self.body.symbol(id);
                write!(f, "{}", crate::cname(symbol.name)).map(|_| idx + 1)
            }
            Instr::Var(id) => write!(f, "{}", self.var_names[id.0 as usize]).map(|_| idx + 1),
            Instr::Field(field_idx) => {
                if precedence < 2 {
                    write!(f, "(")?;
                }

                // Object
                let ty = self.module.inline_ty(self.body.value_ty(idx + 1));
                idx = self.instr(f, idx + 1, 2)?;
                let orco::Type::Struct { fields } = ty else {
                    panic!("trying to access field #{field_idx} on a non-struct type {ty}");
                };

                // Field
                match &fields[field_idx as usize].0 {
                    Some(name) => write!(f, ".{name}")?,
                    None => write!(f, "._{field_idx}")?,
                }

                if precedence < 2 {
                    write!(f, ")")?;
                }

                Ok(idx)
            }
            Instr::Assign => {
                idx = self.instr(f, idx + 1, 15)?;
                write!(f, " = ")?;
                self.instr(f, idx, 16)
            }

            Instr::AcfLabel(label) => {
                write!(f, "{}:", self.label_names[label.0 as usize]).map(|_| idx + 1)
            }
            Instr::AcfJump(label) => {
                write!(f, "jump {}", self.label_names[label.0 as usize]).map(|_| idx + 1)
            }
            Instr::AcfCJump(label) => {
                write!(f, "if (")?;
                idx = self.instr(f, idx + 1, 255)?;
                write!(f, ") jump {}", self.label_names[label.0 as usize])?;
                Ok(idx)
            }

            Instr::Call(args) => {
                idx = self.instr(f, idx + 1, 2)?;
                write!(f, "(")?;
                for i in 0..args {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    idx = self.instr(f, idx, 16)?;
                }
                write!(f, ")").map(|_| idx)
            }

            Instr::Return(has_value) => {
                write!(f, "return")?;
                if has_value {
                    write!(f, " ")?;
                    self.instr(f, idx + 1, 255)
                } else {
                    Ok(idx + 1)
                }
            }

            Instr::Intrinsic(orco::ir::Intrinsic::AggregateLiteral(count)) => {
                let mut value = 0;
                for i in 1..=count {
                    let segment = match self.body.instructions[idx + i as usize] {
                        Instr::IConst(segment, _) => segment.cast_unsigned(),
                        Instr::UConst(segment, _) => segment,
                        instr => panic!("aggregation applied to invalid instruction {instr}"),
                    };
                    value <<= 32;
                    value |= segment as u128;
                }
                use orco::Type;
                match self.body.value_ty(idx) {
                    Type::Integer(size) => {
                        write!(f, "{}{}", value.cast_signed(), int_size_suffix(size))?
                    }
                    Type::Unsigned(size) => write!(f, "{value}u{}", int_size_suffix(size))?,
                    ty => panic!("Invalid type for aggregation: {ty}"),
                }

                Ok(idx + 1 + count as usize)
            }

            Instr::Intrinsic(_) => intrinsics::format(self, f, idx, precedence),
            instr => todo!("{instr}"),
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
                    ty: &var.ty,
                    constant: false,
                    name: Some(&self.var_names[idx])
                }
            )?;
        }

        let mut idx = 0;
        while idx < self.body.instructions.len() {
            if matches!(self.body.instructions[idx], Instr::AcfLabel(..)) {
                idx = self.instr(f, idx, 0)?;
                writeln!(f)?;
                continue;
            }

            write!(f, "  ")?;
            idx = self.instr(f, idx, 0)?;
            writeln!(f, ";")?;
        }

        write!(f, "}}")
    }
}
