pub fn is_thermostat_name(name: &String) -> bool {
    let parts: Vec<&str> = name.split(';').collect();
    if parts.len() != 3 {
        return false;
    }
    
    // Should start with a single digit and end with ";eTRV"
    if !is_single_digit(parts[0]) || parts[2] != "eTRV" {
        return false;
    }

    // Middle part is a single digit, followed by hex bytes separated by :
    let middle_parts: Vec<&str> = parts[1].split(':').collect();
    middle_parts.len() == 6
        && is_single_digit(middle_parts[0])
        && is_hexadecimal_byte(middle_parts[1])
        && is_hexadecimal_byte(middle_parts[2])
        && is_hexadecimal_byte(middle_parts[3])
        && is_hexadecimal_byte(middle_parts[4])
        && is_hexadecimal_byte(middle_parts[5])
}

pub fn stripped_name(name: &String) -> String {
    if !is_thermostat_name(name) {
        panic!("Not a thermostat name: {}", name);
    }
    let mut parts = name.split(';');
    parts.next();
    parts.next().unwrap().to_string()
}

fn is_single_digit(s: &str) -> bool {
    s.len() == 1 && s.chars().all(|c| c.is_ascii_digit())
}

fn is_hexadecimal_byte(s: &str) -> bool {
    s.len() == 2 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_knows_what_is_a_thermostat_name() {
        assert_eq!(true, is_thermostat_name(&"0;0:04:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(true, is_thermostat_name(&"4;0:04:2F:C0:F2:58;eTRV".to_string()));
        assert_eq!(true, is_thermostat_name(&"2;0:04:2F:06:24:DD;eTRV".to_string()));
    }

    #[test]
    fn it_knows_that_thermostat_names_do_not_start_with_non_digits() {
        assert_eq!(false, is_thermostat_name(&"A;0:04:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"a;0:04:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"*;0:04:2F:06:24:D1;eTRV".to_string()));
    }

    #[test]
    #[allow(non_snake_case)]
    fn it_knows_that_thermostat_names_should_end_with_eTRV() {
        assert_eq!(false, is_thermostat_name(&"0;0:04:2F:06:24:D1".to_string()));
        assert_eq!(false, is_thermostat_name(&"4;0:04:2F:C0:F2:58;eTRV3".to_string()));
        assert_eq!(false, is_thermostat_name(&"2;0:04:2F:06:24:DD;3eTRV".to_string()));
    }

    #[test]
    fn it_knows_that_middle_part_should_be_only_colons_and_hexadecimals() {
        assert_eq!(false, is_thermostat_name(&"0;0:G4:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"4;0:_4:2F:C0:F2:58;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"2;0:04;2F:06:24:DD;eTRV".to_string()));
    }

    #[test]
    fn it_knows_that_middle_part_should_be_one_digit_and_five_hex_bytes() {
        assert_eq!(false, is_thermostat_name(&"0;0:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"0;04:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"0;0:0:2F:06:24:D1;eTRV".to_string()));
        assert_eq!(false, is_thermostat_name(&"0;0:04:2F:06:24:D1:D2;eTRV".to_string()));
    }

    #[test]
    fn it_can_trim_thermostat_name() {
        assert_eq!("0:04:2F:06:24:D1", stripped_name(&"0;0:04:2F:06:24:D1;eTRV".to_string()));
    }

}