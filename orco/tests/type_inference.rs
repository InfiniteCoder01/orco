use std::num::NonZeroU16;

mod parser_utils;
use parser_utils::*;

#[test]
fn overflow() {
    {
        make_type_inference!(type_inference, errors);
        let mut big_unsigned = ir::expression::Constant::Integer {
            value: 1 << 127 - 1,
            r#type: ir::Type::IntegerWildcard,
        };
        big_unsigned.infer_types(
            &ir::Type::Int(NonZeroU16::new(16).unwrap()),
            &mut type_inference,
        );
        big_unsigned.finish_and_check_types(dummy_span(), &mut type_inference);
        check!(
            big_unsigned
                == ir::expression::Constant::Integer {
                    value: 1 << 127 - 1,
                    r#type: ir::Type::Int(NonZeroU16::new(16).unwrap())
                }
        );
        check!(errors.is_empty());
    }

    {
        make_type_inference!(type_inference, errors);
        let mut big_unsigned = ir::expression::Constant::Integer {
            value: 1 << 127,
            r#type: ir::Type::IntegerWildcard,
        };
        big_unsigned.infer_types(
            &ir::Type::Int(NonZeroU16::new(16).unwrap()),
            &mut type_inference,
        );
        big_unsigned.finish_and_check_types(dummy_span(), &mut type_inference);
        check!(
            big_unsigned
                == ir::expression::Constant::Integer {
                    value: 1 << 127,
                    r#type: ir::Type::Int(NonZeroU16::new(16).unwrap()),
                }
        );
        check!(errors.len() == 1);
    }
}
