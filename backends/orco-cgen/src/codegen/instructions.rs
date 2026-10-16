use orco::ir::Instr;

impl super::Context<'_> {
    /// Codegen an instruction at `idx` with it's arguments into `f`.
    /// Set precedence to 255 to allow all expressions unparenthesised.
    /// See https://en.cppreference.com/c/language/operator_precedence for other values.
    pub fn instr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        mut idx: usize,
        precedence: u8,
    ) -> Result<usize, std::fmt::Error> {
        use orco::ty::IntegerSize;
        fn int_size_suffix(size: IntegerSize) -> &'static str {
            match size {
                IntegerSize::Bits(bits) if bits.get() > 32 => "ll",
                IntegerSize::Bits(bits) if bits.get() > 16 => "l",
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
                let ty = self
                    .module
                    .inline_ty(self.body.value_ty(self.module, idx + 1));
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
                if crate::ty::is_unit(&self.body.value_ty(self.module, idx + 1)) {
                    // TODO: Skipping this is unsafe...
                    idx = self.body.skip_instr(idx + 1);
                } else {
                    idx = self.instr(f, idx + 1, 15)?;
                    write!(f, " = ")?;
                }
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
                let mut first = true;
                for _ in 0..args {
                    if crate::ty::is_unit(&self.body.value_ty(self.module, idx)) {
                        // TODO: Skipping this is unsafe...
                        idx = self.body.skip_instr(idx);
                        continue;
                    }

                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    idx = self.instr(f, idx, 16)?;
                }
                write!(f, ")").map(|_| idx)
            }

            Instr::Return => {
                write!(f, "return")?;
                if crate::ty::is_unit(&self.body.value_ty(self.module, idx + 1)) {
                    // TODO: Skipping this is unsafe...
                    Ok(self.body.skip_instr(idx + 1))
                } else {
                    write!(f, " ")?;
                    self.instr(f, idx + 1, 255)
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
                match self.body.value_ty(self.module, idx) {
                    Type::Integer(size) => {
                        write!(f, "{}{}", value.cast_signed(), int_size_suffix(size))?
                    }
                    Type::Unsigned(size) => write!(f, "{value}u{}", int_size_suffix(size))?,
                    ty => panic!("Invalid type for aggregation: {ty}"),
                }

                Ok(idx + 1 + count as usize)
            }

            Instr::Intrinsic(_) => super::intrinsics::format(self, f, idx, precedence),

            Instr::Cast(ty) => {
                if precedence < 2 {
                    write!(f, "(")?;
                }

                write!(
                    f,
                    "({})",
                    crate::FmtType {
                        ty: self.body.interned_ty(ty),
                        constant: false,
                        name: None
                    }
                )?;
                idx = self.instr(f, idx + 1, 2)?;

                if precedence < 2 {
                    write!(f, ")")?;
                }

                Ok(idx)
            }

            instr => todo!("{instr}"),
        }
    }
}
