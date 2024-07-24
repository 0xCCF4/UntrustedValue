use untrusted_value::UntrustedValue;

#[no_mangle]
fn works() {
    let insecure_env = std::env::var("TEST");

    // do some stuff in between
    println!("waiting...");
    std::thread::sleep(std::time::Duration::from_secs(10));

    let secure_env = UntrustedValue::from(insecure_env);

    println!("{:?}", secure_env.use_untrusted_value())
}

#[no_mangle]
fn fails() {
    let insecure_env = std::env::var("TEST");

    println!("{:?}", insecure_env)
}

fn main() {
    works();
    fails();
}
