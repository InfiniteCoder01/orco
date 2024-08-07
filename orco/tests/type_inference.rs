use std::num::NonZeroU16;

mod parser_utils;
use parser_utils::*;

#[test]
fn overflow() {
    {
        make_type_inference!(type_inference, errors);
        let mut big_unsigned = ir::expression::Constant::Integer {
            value: (1 << 127) - 1,
            r#type: ir::Type::IntegerWildcard,
            metadata: Box::new(()),
        };
        let r#type = big_unsigned.infer_types(&mut type_inference);
        type_inference.equate(&r#type, &ir::Type::Int(NonZeroU16::new(16).unwrap()));
        big_unsigned.finish_and_check_types(&None, &mut type_inference);
        check!(
            big_unsigned
                == ir::expression::Constant::Integer {
                    value: (1 << 127) - 1,
                    r#type: ir::Type::Int(NonZeroU16::new(16).unwrap()),
                    metadata: Box::new(()),
                }
        );
        check!(errors.is_empty());
    }

    {
        make_type_inference!(type_inference, errors);
        let mut big_unsigned = ir::expression::Constant::Integer {
            value: 1 << 127,
            r#type: ir::Type::IntegerWildcard,
            metadata: Box::new(()),
        };
        let r#type = big_unsigned.infer_types(&mut type_inference);
        type_inference.equate(&r#type, &ir::Type::Int(NonZeroU16::new(16).unwrap()));
        big_unsigned.finish_and_check_types(&None, &mut type_inference);
        check!(
            big_unsigned
                == ir::expression::Constant::Integer {
                    value: 1 << 127,
                    r#type: ir::Type::Int(NonZeroU16::new(16).unwrap()),
                    metadata: Box::new(()),
                }
        );
        check!(errors.len() == 1);
    }
}
