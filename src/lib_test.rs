mod tests {
    use crate::parse;

    #[test]
    fn table_driven_supported_forms() {
        let cases = [
            ("every minute", "* * * * *"),
            ("every 15 minutes", "*/15 * * * *"),
            ("every hour", "0 * * * *"),
            ("every 12 hours", "0 */12 * * *"),
            ("every day", "0 0 * * *"),
            ("every 1 day", "0 0 * * *"),
            ("every day at 03:30", "30 3 * * *"),
            ("every 12:00", "0 12 * * *"),
            ("every monday", "0 0 * * 1"),
            ("every friday at 18:00", "0 18 * * 5"),
            ("every mon, wed and fri at 06:30", "30 6 * * 1,3,5"),
            ("every weekday at 09:00", "0 9 * * 1-5"),
            ("every weekend at 10:00", "0 10 * * 6,0"),
            ("every month", "0 0 1 * *"),
            ("every month on the 1st at 03:00", "0 3 1 * *"),
        ];

        for (input, expected_cron) in cases {
            let sched = parse(input).unwrap_or_else(|e| panic!("failed to parse `{input}`: {e}"));
            assert_eq!(sched.cron, expected_cron, "mismatch for `{input}`");
        }
    }

    #[test]
    fn whitespace_and_casing_variations() {
        let sched = parse("  Every   FRIDAY at 18:00 ").expect("parse");
        assert_eq!(sched.cron, "0 18 * * 5");

        let sched2 = parse("EVERY 15 MINUTES").expect("parse");
        assert_eq!(sched2.cron, "*/15 * * * *");
    }

    #[test]
    fn am_pm_time_edge_cases() {
        assert_eq!(parse("every 12am").unwrap().cron, "0 0 * * *");
        assert_eq!(parse("every 12pm").unwrap().cron, "0 12 * * *");
        assert_eq!(parse("every 6pm").unwrap().cron, "0 18 * * *");
        assert_eq!(parse("every 6:30pm").unwrap().cron, "30 18 * * *");
        assert_eq!(parse("every 12:30am").unwrap().cron, "30 0 * * *");
        assert_eq!(parse("every 12:30pm").unwrap().cron, "30 12 * * *");
    }

    #[test]
    fn negative_tests_with_asserted_error_messages() {
        // every 5 hours
        let err = parse("every 5 hours").unwrap_err();
        assert!(err.to_string().contains("does not divide 24"), "{err}");

        // every 7 minutes
        let err = parse("every 7 minutes").unwrap_err();
        assert!(err.to_string().contains("does not divide 60"), "{err}");

        // every 2 days
        let err = parse("every 2 days").unwrap_err();
        assert!(
            err.to_string().contains("every 2 days is not supported"),
            "{err}"
        );

        // every month on the 31st
        let err = parse("every month on the 31st").unwrap_err();
        assert!(
            err.to_string().contains("must be between 1 and 28"),
            "{err}"
        );

        // every someday
        let err = parse("every someday").unwrap_err();
        assert!(
            err.to_string().contains("unknown day name `someday`"),
            "{err}"
        );

        // every 25:00
        let err = parse("every 25:00").unwrap_err();
        assert!(err.to_string().contains("invalid time `25:00`"), "{err}");

        // every friday at 18:60
        let err = parse("every friday at 18:60").unwrap_err();
        assert!(err.to_string().contains("invalid time `18:60`"), "{err}");

        // empty string
        let err = parse("").unwrap_err();
        assert!(err.to_string().contains("empty"), "{err}");

        // missing every
        let err = parse("friday at 18:00").unwrap_err();
        assert!(err.to_string().contains("must begin with 'every'"), "{err}");

        // every 0 minutes
        let err = parse("every 0 minutes").unwrap_err();
        assert!(err.to_string().contains("at least 1"), "{err}");

        // every 0 hours
        let err = parse("every 0 hours").unwrap_err();
        assert!(err.to_string().contains("at least 1"), "{err}");

        // every month on the 29th
        let err = parse("every month on the 29th").unwrap_err();
        assert!(
            err.to_string().contains("must be between 1 and 28"),
            "{err}"
        );
    }

    #[test]
    fn weekend_and_weekday_defaults() {
        assert_eq!(parse("every weekend").unwrap().cron, "0 0 * * 6,0");
        assert_eq!(parse("every weekday").unwrap().cron, "0 0 * * 1-5");
        assert_eq!(
            parse("every weekend at 10:00").unwrap().cron,
            "0 10 * * 6,0"
        );
        assert_eq!(parse("every weekday at 09:00").unwrap().cron, "0 9 * * 1-5");
    }

    #[test]
    fn property_test_all_valid_crons_parse_in_croner_with_5_fields() {
        use croner::parser::{CronParser, Seconds};

        let samples = [
            "every minute",
            "every 1 minute",
            "every 2 minutes",
            "every 5 minutes",
            "every 10 minutes",
            "every 15 minutes",
            "every 20 minutes",
            "every 30 minutes",
            "every hour",
            "every 1 hour",
            "every 2 hours",
            "every 3 hours",
            "every 4 hours",
            "every 6 hours",
            "every 8 hours",
            "every 12 hours",
            "every day",
            "every 1 day",
            "every day at 00:00",
            "every day at 03:30",
            "every day at 23:59",
            "every 12:00",
            "every 6pm",
            "every 12am",
            "every 12pm",
            "every monday",
            "every mondays",
            "every tuesday",
            "every wednesday",
            "every thursday",
            "every friday",
            "every saturday",
            "every sunday",
            "every friday at 18:00",
            "every mon, wed and fri at 06:30",
            "every weekday at 09:00",
            "every weekend at 10:00",
            "every month",
            "every month on the 1st at 03:00",
            "every month on the 15th at 12:00",
            "every month on 28th at 23:59",
        ];

        for sample in samples {
            let sched = parse(sample).expect(sample);
            let fields: Vec<&str> = sched.cron.split_whitespace().collect();
            assert_eq!(
                fields.len(),
                5,
                "schedule `{sample}` cron `{}` must have exactly 5 fields",
                sched.cron
            );

            // Verify croner parses it cleanly
            let parsed = CronParser::builder()
                .seconds(Seconds::Optional)
                .build()
                .parse(&sched.cron);

            assert!(
                parsed.is_ok(),
                "croner failed to parse `{}` from sample `{sample}`: {:?}",
                sched.cron,
                parsed.err()
            );
        }
    }
    #[test]
    fn yearly_expressions() {
        let cases = [
            ("every 1st of february", "0 0 1 2 *"),
            ("every 15th of december", "0 0 15 12 *"),
            ("every 28th of february", "0 0 28 2 *"),
            ("every 31st of january", "0 0 31 1 *"),
            ("every 30th of april", "0 0 30 4 *"),
            ("every 1st of jan", "0 0 1 1 *"),
            ("every 15th of dec at 09:00", "0 9 15 12 *"),
            ("every 1st of february at 00:00", "0 0 1 2 *"),
        ];

        for (input, expected_cron) in cases {
            let sched = parse(input).unwrap_or_else(|e| panic!("failed to parse `{input}`: {e}"));
            assert_eq!(sched.cron, expected_cron, "mismatch for `{input}`");
        }

        // Feb 29th is invalid (we cap at 28 to avoid leap-year ambiguity)
        let err = parse("every 29th of february").unwrap_err();
        assert!(
            err.to_string().contains("out of range for February"),
            "{err}"
        );

        // Apr has only 30 days
        let err = parse("every 31st of april").unwrap_err();
        assert!(err.to_string().contains("out of range for April"), "{err}");
    }
}
