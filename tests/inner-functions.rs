use fehler::throws;

type Error = ();

#[throws]
fn inner_function() {
    fn foo() {
    }
}

#[throws]
fn fn_parameters(_: fn()) {
}

#[throws]
fn fn_type_alias() {
    type X = fn();
}

#[throws]
fn type_ascription() {
    let _: fn() = panic!();
}
