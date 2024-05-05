pub fn scale_value(value: f32, ranges: &[(f32, f32)]) -> f32 {
    let mut ranges = Vec::from(ranges);
    ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Find the largest value that is smaller
    let mut from = (f32::MIN, None);

    // Find the smallest value that is larger
    let mut to = (f32::MAX, None);

    for (map_from, map_to) in ranges {
        if map_from <= value && map_from > from.0 {
            from = (map_from, Some(map_to));
        }

        if map_from >= value && map_from < to.0 {
            to = (map_from, Some(map_to));
        }
    }

    match (from, to) {
        // No value is smaller (given value is below all other values)
        ((_, None), (_, Some(v))) => v,
        // No value is larger (given value exceeds all values)
        ((_, Some(v)), (_, None)) => v,
        ((f_in, Some(f_out)), (t_in, Some(t_out))) => {
            // The length of the input range
            let range_in = t_in - f_in;
            // The lange of the output range
            let range_out = t_out - f_out;
            assert!(
                range_in >= 0.,
                "input range {} isn't larger equal than 0",
                range_in
            );
            assert!(
                range_out >= 0.,
                "output range {} isn't larger equal than 0",
                range_out
            );
            if range_in > 0.0 {
                // This is the offset within the input range
                let offset = value - f_in;
                assert!(
                    offset <= range_in,
                    "Offset {} should not be larger than input range {}",
                    offset,
                    range_in
                );
                assert!(offset >= 0.);
                // Factor to scale between input and output range
                let factor = range_out / range_in;
                f_out + (offset * factor)
            } else {
                f_out
            }
        }
        (_, _) => f32::NAN,
    }
}

#[test]
pub fn test_scale_value() {
    assert_eq!(scale_value(10., &[(10., 1.)]), 1.);
    assert_eq!(scale_value(5., &[(10., 1.)]), 1.);
    assert_eq!(scale_value(30., &[(10., 1.)]), 1.);
    assert_eq!(scale_value(5., &[(10., 1.), (20., 2.)]), 1.);
    assert_eq!(scale_value(30., &[(10., 1.), (20., 2.)]), 2.);
    assert_eq!(scale_value(20., &[(10., 1.), (20., 2.)]), 2.);
    assert_eq!(scale_value(15., &[(10., 1.), (20., 2.)]), 1.5);
    assert_eq!(
        scale_value(21807.204688249418, &[(0., 0.), (30000., 5.)]),
        3.6345341
    );
}
