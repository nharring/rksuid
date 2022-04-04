extern crate rksuid;

#[cfg(test)]
mod tests {
    use rksuid::Ksuid;

    fn build_segment_ksuid() -> Ksuid {
        Ksuid::new_with_timestamp_and_payload(107608047, 0xB5A1CD34B5F99D1154FB6853345C9735)
    }
    // Compat tests with ksuids generated by the segment go code
    #[test]
    fn test_serialize_compat_with_segment_ksuid() {
        // From github.com/segmentio/ksuid#inspect-the-components-of-a-ksuid
        // REPRESENTATION:
        //   String: 0ujtsYcgvSTl8PAuAdqWYSMnLOv
        //      Raw: 0669F7EFB5A1CD34B5F99D1154FB6853345C9735
        // COMPONENTS:
        //        Time: 2017-10-09 21:00:47 -0700 PDT
        //   Timestamp: 107608047
        //     Payload: B5A1CD34B5F99D1154FB6853345C9735
        let segment_serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
        let ksuid_known_good = build_segment_ksuid();
        let test_serialized = ksuid_known_good.serialize();
        assert_eq!(segment_serialized, test_serialized);
    }

    #[test]
    fn test_deserialize_compat_with_segment_ksuid() {
        let segment_serialized = "0ujtsYcgvSTl8PAuAdqWYSMnLOv";
        let ksuid = rksuid::deserialize(segment_serialized);
        let known_good = build_segment_ksuid();
        assert_eq!(ksuid, known_good);
    }
}
