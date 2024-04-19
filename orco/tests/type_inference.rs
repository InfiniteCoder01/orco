use std::num::NonZeroU16;

use orco::ir::*;

#[test]
fn overflow() {
    let big_value = 170141183460469231731687303715884105728;
    let mut big_unsigned = expression::Constant::UnsignedInteger {
        value: big_value - 1,
        size: None,
    };
    big_unsigned.infer_types(&Type::Int(NonZeroU16::new(16).unwrap()));
    assert_eq!(
        big_unsigned,
        expression::Constant::SignedInteger {
            value: (big_value - 1) as _,
            size: Some(16)
        }
    );
    let mut big_unsigned = expression::Constant::UnsignedInteger {
        value: big_value,
        size: None,
    };
    big_unsigned.infer_types(&Type::Int(NonZeroU16::new(16).unwrap()));
    assert_eq!(
        big_unsigned,
        expression::Constant::UnsignedInteger {
            value: big_value,
            size: None,
        }
    );
}
