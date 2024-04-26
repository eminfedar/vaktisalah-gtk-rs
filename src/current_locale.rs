// Source: https://github.com/i509VCB/current_locale/blob/master/src/unix.rs

use std::env;

pub(crate) fn current_locale() -> String {
    // Unix uses the LANG environment variable to store the locale
    match env::var("LANG") {
        Ok(raw_lang) => {
            // Unset locale - C ANSI standards say default to en-US
            if raw_lang == "C" {
                "en-US".to_owned()
            } else {
                normalize_to_ietf(&raw_lang)
            }
        }

        Err(_e) => "en-US".to_owned(),
    }
}

/// Normalizes a unix locale value to an ietf compliant language code.
fn normalize_to_ietf(raw: &str) -> String {
    /*
     * Find one of the following to split off the lang code:
     * First index of `.` as in `en_US.UTF_8`
     * A space which separates generic code from char set.
     * Terminate at an `@` which specifies a locale at a specific location
     */
    if let Some(pos) = raw.find(|c| c == ' ' || c == '.') {
        let (raw_lang_code, _) = raw.split_at(pos);
        let result = raw_lang_code.replace('_', "-");

        // Finally replace underscores with `-` and drop everything after an `@`
        return result.split('@').next().unwrap().to_string();
    }

    "en-US".to_owned()
}
