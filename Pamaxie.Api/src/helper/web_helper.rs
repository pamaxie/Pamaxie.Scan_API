use std::env;

pub fn get_env_variable(env_var_name: String, alternate_value: String) -> String{
    let env_value = env::var(&env_var_name);

    return if env_value.is_err() || env_value.as_ref().unwrap().is_empty()
    {
        alternate_value
    } else
    {
        env_value.unwrap()
    }
}