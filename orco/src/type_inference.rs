/// This struct holds type inference state.
/// On the first pass it's passed to every node
/// as a mutable reference and gets filled with
/// type variables, on the second pass it's passed
/// as an immutable reference and is used to
/// finalise the types and send out diagnostics
pub struct TypeInferenceContext {
    //
}
