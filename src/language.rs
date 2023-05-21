use once_cell::sync::Lazy;
use std::collections::HashMap;

pub fn get_language_code(language: &str) -> &str {
    static LANGUAGES_TO_CODE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
        let data = vec![
            ("afar", "AA"),
            ("abkhaz", "AB"),
            ("avestan", "AE"),
            ("afrikaans", "AF"),
            ("akan", "AK"),
            ("amharic", "AM"),
            ("aragonese", "AN"),
            ("arabic", "AR"),
            ("assamese", "AS"),
            ("avaric", "AV"),
            ("aymara", "AY"),
            ("azerbaijani", "AZ"),
            ("bashkir", "BA"),
            ("belarusian", "BE"),
            ("bulgarian", "BG"),
            ("bislama", "BI"),
            ("bambara", "BM"),
            ("bengali", "BN"),
            ("tibetan", "BO"),
            ("breton", "BR"),
            ("bosnian", "BS"),
            ("catalan", "CA"),
            ("chechen", "CE"),
            ("chamorro", "CH"),
            ("corsican", "CO"),
            ("cree", "CR"),
            ("czech", "CS"),
            ("old church slavonic", "CU"),
            ("chuvash", "CV"),
            ("welsh", "CY"),
            ("danish", "DA"),
            ("german", "DE"),
            ("divehi", "DV"),
            ("dzongkha", "DZ"),
            ("ewe", "EE"),
            ("greek", "EL"),
            ("english", "EN"),
            ("esperanto", "EO"),
            ("spanish", "ES"),
            ("estonian", "ET"),
            ("basque", "EU"),
            ("persian", "FA"),
            ("fula", "FF"),
            ("finnish", "FI"),
            ("fijian", "FJ"),
            ("faroese", "FO"),
            ("french", "FR"),
            ("western frisian", "FY"),
            ("irish", "GA"),
            ("scottish gaelic", "GD"),
            ("galician", "GL"),
            ("guaraní", "GN"),
            ("gujarati", "GU"),
            ("manx", "GV"),
            ("hausa", "HA"),
            ("hebrew", "HE"),
            ("hindi", "HI"),
            ("hiri motu", "HO"),
            ("croatian", "HR"),
            ("haitian", "HT"),
            ("hungarian", "HU"),
            ("armenian", "HY"),
            ("herero", "HZ"),
            ("interlingua", "IA"),
            ("indonesian", "ID"),
            ("interlingue", "IE"),
            ("igbo", "IG"),
            ("nuosu", "II"),
            ("inupiaq", "IK"),
            ("ido", "IO"),
            ("icelandic", "IS"),
            ("italian", "IT"),
            ("inuktitut", "IU"),
            ("japanese", "JA"),
            ("javanese", "JV"),
            ("georgian", "KA"),
            ("kongo", "KG"),
            ("kikuyu", "KI"),
            ("kwanyama", "KJ"),
            ("kazakh", "KK"),
            ("kalaallisut", "KL"),
            ("khmer", "KM"),
            ("kannada", "KN"),
            ("korean", "KO"),
            ("kanuri", "KR"),
            ("kashmiri", "KS"),
            ("kurdish", "KU"),
            ("komi", "KV"),
            ("cornish", "KW"),
            ("kyrgyz", "KY"),
            ("latin", "LA"),
            ("luxembourgish", "LB"),
            ("ganda", "LG"),
            ("limburgish", "LI"),
            ("lingala", "LN"),
            ("lao", "LO"),
            ("lithuanian", "LT"),
            ("luba-katanga", "LU"),
            ("latvian", "LV"),
            ("malagasy", "MG"),
            ("marshallese", "MH"),
            ("māori", "MI"),
            ("macedonian", "MK"),
            ("malayalam", "ML"),
            ("mongolian", "MN"),
            ("marathi", "MR"),
            ("malay", "MS"),
            ("maltese", "MT"),
            ("burmese", "MY"),
            ("nauru", "NA"),
            ("norwegian bokmål", "NB"),
            ("northern ndebele", "ND"),
            ("nepali", "NE"),
            ("ndonga", "NG"),
            ("dutch", "NL"),
            ("norwegian nynorsk", "NN"),
            ("norwegian", "NO"),
            ("southern ndebele", "NR"),
            ("navajo", "NV"),
            ("chichewa", "NY"),
            ("occitan", "OC"),
            ("ojibwe", "OJ"),
            ("oromo", "OM"),
            ("oriya", "OR"),
            ("ossetian", "OS"),
            ("panjabi", "PA"),
            ("pāli", "PI"),
            ("polish", "PL"),
            ("pashto", "PS"),
            ("portuguese", "PT"),
            ("quechua", "QU"),
            ("romansh", "RM"),
            ("kirundi", "RN"),
            ("romanian", "RO"),
            ("russian", "RU"),
            ("kinyarwanda", "RW"),
            ("sanskrit", "SA"),
            ("sardinian", "SC"),
            ("sindhi", "SD"),
            ("northern sami", "SE"),
            ("sango", "SG"),
            ("sinhala", "SI"),
            ("slovak", "SK"),
            ("slovenian", "SL"),
            ("samoan", "SM"),
            ("shona", "SN"),
            ("somali", "SO"),
            ("albanian", "SQ"),
            ("serbian", "SR"),
            ("swati", "SS"),
            ("southern sotho", "ST"),
            ("sundanese", "SU"),
            ("swedish", "SV"),
            ("swahili", "SW"),
            ("tamil", "TA"),
            ("telugu", "TE"),
            ("tajik", "TG"),
            ("thai", "TH"),
            ("tigrinya", "TI"),
            ("turkmen", "TK"),
            ("tagalog", "TL"),
            ("tswana", "TN"),
            ("tonga", "TO"),
            ("turkish", "TR"),
            ("tsonga", "TS"),
            ("tatar", "TT"),
            ("twi", "TW"),
            ("tahitian", "TY"),
            ("uyghur", "UG"),
            ("ukrainian", "UK"),
            ("urdu", "UR"),
            ("uzbek", "UZ"),
            ("venda", "VE"),
            ("vietnamese", "VI"),
            ("volapük", "VO"),
            ("walloon", "WA"),
            ("wolof", "WO"),
            ("xhosa", "XH"),
            ("yiddish", "YI"),
            ("yoruba", "YO"),
            ("zhuang", "ZA"),
            ("chinese", "ZH"),
            ("zulu", "ZU"),
        ];
        data.iter().cloned().collect()
    });

    LANGUAGES_TO_CODE.get(language).unwrap_or(&"")
}
