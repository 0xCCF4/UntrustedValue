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

#[no_mangle]
fn complicated_nonsense_function(mut arg: String) -> String {
    // dont mind the horrible code, used to check the data flow algorithm ;)

    let count = arg.chars().filter(|c| *c == '.').count();

    loop {
        for i in 0..arg.len() {
            if let Some(index) = arg.find(".") {
                arg.remove(index);
                arg.push('+');
            }
            let ith = arg.chars().nth(i).unwrap();
            if ith == '#' {
                return arg.to_lowercase();
            } else if ith == '-' {
                arg = arg.to_uppercase();
            } else if ith == '_' {
                arg = arg.to_lowercase();
            }
        }
        if !arg.contains(".") {
            break;
        }
    }

    arg.repeat(count)
}

#[no_mangle]
fn nested_function(mut input: usize) -> usize {
    #[no_mangle]
    fn inner_function(unsafe_arg: &mut usize) {
        *unsafe_arg = 10;
    }

    inner_function(&mut input);

    input
}

struct ABC {
    value: i32,
}
#[no_mangle]
fn test_rename(q: ABC) -> ABC {
    let mut x = q;
    let mut y = x;
    y.value = 10;
    let z = y;

    z
}

fn main() {
    works();
    fails();
}
