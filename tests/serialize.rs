extern crate rksuid;

#[cfg(test)]
mod tests {
    use rksuid::Ksuid;

    // SerDe tests
    #[test]
    fn test_serialize_with_random_data_returns_right_length() {
        let ksuid = Ksuid::new();
        let serialized = ksuid.serialize();
        assert_eq!(serialized.char_indices().count(), 27);
    }
    #[test]
    fn test_serialize_deserialize() {
        let ksuid = Ksuid::new();
        let serialized = ksuid.serialize();
        let ksuid2 = rksuid::deserialize(&serialized);
        assert_eq!(ksuid, ksuid2);
    }
    #[test]
    fn test_get_formatted_lines() {
        let ksuid = rksuid::deserialize("0ujtsYcgvSTl8PAuAdqWYSMnLOv");
        let formatted = ksuid.get_formatted_lines();
        assert!(!formatted.is_empty());
        let timestamp_line = "\tTimestamp: 107608047";
        assert_eq!(formatted[5], timestamp_line);
        let raw_line = "\tRaw: 669F7EFB5A1CD34B5F99D1154FB6853";
        assert_eq!(formatted[2], raw_line);
    }
}
