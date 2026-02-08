//! Emoji code mappings from name to Unicode character.
//!
//! This is a data file — auto-generated from Python rich's `_emoji_codes.py`.

use std::sync::LazyLock;
use std::collections::HashMap;

/// Lookup table mapping emoji names to their Unicode character sequences.
pub static EMOJI: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(3608);
    m.insert("1st_place_medal", "\u{1F947}");
    m.insert("2nd_place_medal", "\u{1F948}");
    m.insert("3rd_place_medal", "\u{1F949}");
    m.insert("ab_button_(blood_type)", "\u{1F18E}");
    m.insert("atm_sign", "\u{1F3E7}");
    m.insert("a_button_(blood_type)", "\u{1F170}");
    m.insert("afghanistan", "\u{1F1E6}\u{1F1EB}");
    m.insert("albania", "\u{1F1E6}\u{1F1F1}");
    m.insert("algeria", "\u{1F1E9}\u{1F1FF}");
    m.insert("american_samoa", "\u{1F1E6}\u{1F1F8}");
    m.insert("andorra", "\u{1F1E6}\u{1F1E9}");
    m.insert("angola", "\u{1F1E6}\u{1F1F4}");
    m.insert("anguilla", "\u{1F1E6}\u{1F1EE}");
    m.insert("antarctica", "\u{1F1E6}\u{1F1F6}");
    m.insert("antigua_&_barbuda", "\u{1F1E6}\u{1F1EC}");
    m.insert("aquarius", "\u{2652}");
    m.insert("argentina", "\u{1F1E6}\u{1F1F7}");
    m.insert("aries", "\u{2648}");
    m.insert("armenia", "\u{1F1E6}\u{1F1F2}");
    m.insert("aruba", "\u{1F1E6}\u{1F1FC}");
    m.insert("ascension_island", "\u{1F1E6}\u{1F1E8}");
    m.insert("australia", "\u{1F1E6}\u{1F1FA}");
    m.insert("austria", "\u{1F1E6}\u{1F1F9}");
    m.insert("azerbaijan", "\u{1F1E6}\u{1F1FF}");
    m.insert("back_arrow", "\u{1F519}");
    m.insert("b_button_(blood_type)", "\u{1F171}");
    m.insert("bahamas", "\u{1F1E7}\u{1F1F8}");
    m.insert("bahrain", "\u{1F1E7}\u{1F1ED}");
    m.insert("bangladesh", "\u{1F1E7}\u{1F1E9}");
    m.insert("barbados", "\u{1F1E7}\u{1F1E7}");
    m.insert("belarus", "\u{1F1E7}\u{1F1FE}");
    m.insert("belgium", "\u{1F1E7}\u{1F1EA}");
    m.insert("belize", "\u{1F1E7}\u{1F1FF}");
    m.insert("benin", "\u{1F1E7}\u{1F1EF}");
    m.insert("bermuda", "\u{1F1E7}\u{1F1F2}");
    m.insert("bhutan", "\u{1F1E7}\u{1F1F9}");
    m.insert("bolivia", "\u{1F1E7}\u{1F1F4}");
    m.insert("bosnia_&_herzegovina", "\u{1F1E7}\u{1F1E6}");
    m.insert("botswana", "\u{1F1E7}\u{1F1FC}");
    m.insert("bouvet_island", "\u{1F1E7}\u{1F1FB}");
    m.insert("brazil", "\u{1F1E7}\u{1F1F7}");
    m.insert("british_indian_ocean_territory", "\u{1F1EE}\u{1F1F4}");
    m.insert("british_virgin_islands", "\u{1F1FB}\u{1F1EC}");
    m.insert("brunei", "\u{1F1E7}\u{1F1F3}");
    m.insert("bulgaria", "\u{1F1E7}\u{1F1EC}");
    m.insert("burkina_faso", "\u{1F1E7}\u{1F1EB}");
    m.insert("burundi", "\u{1F1E7}\u{1F1EE}");
    m.insert("cl_button", "\u{1F191}");
    m.insert("cool_button", "\u{1F192}");
    m.insert("cambodia", "\u{1F1F0}\u{1F1ED}");
    m.insert("cameroon", "\u{1F1E8}\u{1F1F2}");
    m.insert("canada", "\u{1F1E8}\u{1F1E6}");
    m.insert("canary_islands", "\u{1F1EE}\u{1F1E8}");
    m.insert("cancer", "\u{264B}");
    m.insert("cape_verde", "\u{1F1E8}\u{1F1FB}");
    m.insert("capricorn", "\u{2651}");
    m.insert("caribbean_netherlands", "\u{1F1E7}\u{1F1F6}");
    m.insert("cayman_islands", "\u{1F1F0}\u{1F1FE}");
    m.insert("central_african_republic", "\u{1F1E8}\u{1F1EB}");
    m.insert("ceuta_&_melilla", "\u{1F1EA}\u{1F1E6}");
    m.insert("chad", "\u{1F1F9}\u{1F1E9}");
    m.insert("chile", "\u{1F1E8}\u{1F1F1}");
    m.insert("china", "\u{1F1E8}\u{1F1F3}");
    m.insert("christmas_island", "\u{1F1E8}\u{1F1FD}");
    m.insert("christmas_tree", "\u{1F384}");
    m.insert("clipperton_island", "\u{1F1E8}\u{1F1F5}");
    m.insert("cocos_(keeling)_islands", "\u{1F1E8}\u{1F1E8}");
    m.insert("colombia", "\u{1F1E8}\u{1F1F4}");
    m.insert("comoros", "\u{1F1F0}\u{1F1F2}");
    m.insert("congo_-_brazzaville", "\u{1F1E8}\u{1F1EC}");
    m.insert("congo_-_kinshasa", "\u{1F1E8}\u{1F1E9}");
    m.insert("cook_islands", "\u{1F1E8}\u{1F1F0}");
    m.insert("costa_rica", "\u{1F1E8}\u{1F1F7}");
    m.insert("croatia", "\u{1F1ED}\u{1F1F7}");
    m.insert("cuba", "\u{1F1E8}\u{1F1FA}");
    m.insert("curaçao", "\u{1F1E8}\u{1F1FC}");
    m.insert("cyprus", "\u{1F1E8}\u{1F1FE}");
    m.insert("czechia", "\u{1F1E8}\u{1F1FF}");
    m.insert("côte_d’ivoire", "\u{1F1E8}\u{1F1EE}");
    m.insert("denmark", "\u{1F1E9}\u{1F1F0}");
    m.insert("diego_garcia", "\u{1F1E9}\u{1F1EC}");
    m.insert("djibouti", "\u{1F1E9}\u{1F1EF}");
    m.insert("dominica", "\u{1F1E9}\u{1F1F2}");
    m.insert("dominican_republic", "\u{1F1E9}\u{1F1F4}");
    m.insert("end_arrow", "\u{1F51A}");
    m.insert("ecuador", "\u{1F1EA}\u{1F1E8}");
    m.insert("egypt", "\u{1F1EA}\u{1F1EC}");
    m.insert("el_salvador", "\u{1F1F8}\u{1F1FB}");
    m.insert(
        "england",
        "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}",
    );
    m.insert("equatorial_guinea", "\u{1F1EC}\u{1F1F6}");
    m.insert("eritrea", "\u{1F1EA}\u{1F1F7}");
    m.insert("estonia", "\u{1F1EA}\u{1F1EA}");
    m.insert("ethiopia", "\u{1F1EA}\u{1F1F9}");
    m.insert("european_union", "\u{1F1EA}\u{1F1FA}");
    m.insert("free_button", "\u{1F193}");
    m.insert("falkland_islands", "\u{1F1EB}\u{1F1F0}");
    m.insert("faroe_islands", "\u{1F1EB}\u{1F1F4}");
    m.insert("fiji", "\u{1F1EB}\u{1F1EF}");
    m.insert("finland", "\u{1F1EB}\u{1F1EE}");
    m.insert("france", "\u{1F1EB}\u{1F1F7}");
    m.insert("french_guiana", "\u{1F1EC}\u{1F1EB}");
    m.insert("french_polynesia", "\u{1F1F5}\u{1F1EB}");
    m.insert("french_southern_territories", "\u{1F1F9}\u{1F1EB}");
    m.insert("gabon", "\u{1F1EC}\u{1F1E6}");
    m.insert("gambia", "\u{1F1EC}\u{1F1F2}");
    m.insert("gemini", "\u{264A}");
    m.insert("georgia", "\u{1F1EC}\u{1F1EA}");
    m.insert("germany", "\u{1F1E9}\u{1F1EA}");
    m.insert("ghana", "\u{1F1EC}\u{1F1ED}");
    m.insert("gibraltar", "\u{1F1EC}\u{1F1EE}");
    m.insert("greece", "\u{1F1EC}\u{1F1F7}");
    m.insert("greenland", "\u{1F1EC}\u{1F1F1}");
    m.insert("grenada", "\u{1F1EC}\u{1F1E9}");
    m.insert("guadeloupe", "\u{1F1EC}\u{1F1F5}");
    m.insert("guam", "\u{1F1EC}\u{1F1FA}");
    m.insert("guatemala", "\u{1F1EC}\u{1F1F9}");
    m.insert("guernsey", "\u{1F1EC}\u{1F1EC}");
    m.insert("guinea", "\u{1F1EC}\u{1F1F3}");
    m.insert("guinea-bissau", "\u{1F1EC}\u{1F1FC}");
    m.insert("guyana", "\u{1F1EC}\u{1F1FE}");
    m.insert("haiti", "\u{1F1ED}\u{1F1F9}");
    m.insert("heard_&_mcdonald_islands", "\u{1F1ED}\u{1F1F2}");
    m.insert("honduras", "\u{1F1ED}\u{1F1F3}");
    m.insert("hong_kong_sar_china", "\u{1F1ED}\u{1F1F0}");
    m.insert("hungary", "\u{1F1ED}\u{1F1FA}");
    m.insert("id_button", "\u{1F194}");
    m.insert("iceland", "\u{1F1EE}\u{1F1F8}");
    m.insert("india", "\u{1F1EE}\u{1F1F3}");
    m.insert("indonesia", "\u{1F1EE}\u{1F1E9}");
    m.insert("iran", "\u{1F1EE}\u{1F1F7}");
    m.insert("iraq", "\u{1F1EE}\u{1F1F6}");
    m.insert("ireland", "\u{1F1EE}\u{1F1EA}");
    m.insert("isle_of_man", "\u{1F1EE}\u{1F1F2}");
    m.insert("israel", "\u{1F1EE}\u{1F1F1}");
    m.insert("italy", "\u{1F1EE}\u{1F1F9}");
    m.insert("jamaica", "\u{1F1EF}\u{1F1F2}");
    m.insert("japan", "\u{1F5FE}");
    m.insert("japanese_acceptable_button", "\u{1F251}");
    m.insert("japanese_application_button", "\u{1F238}");
    m.insert("japanese_bargain_button", "\u{1F250}");
    m.insert("japanese_castle", "\u{1F3EF}");
    m.insert("japanese_congratulations_button", "\u{3297}");
    m.insert("japanese_discount_button", "\u{1F239}");
    m.insert("japanese_dolls", "\u{1F38E}");
    m.insert("japanese_free_of_charge_button", "\u{1F21A}");
    m.insert("japanese_here_button", "\u{1F201}");
    m.insert("japanese_monthly_amount_button", "\u{1F237}");
    m.insert("japanese_no_vacancy_button", "\u{1F235}");
    m.insert("japanese_not_free_of_charge_button", "\u{1F236}");
    m.insert("japanese_open_for_business_button", "\u{1F23A}");
    m.insert("japanese_passing_grade_button", "\u{1F234}");
    m.insert("japanese_post_office", "\u{1F3E3}");
    m.insert("japanese_prohibited_button", "\u{1F232}");
    m.insert("japanese_reserved_button", "\u{1F22F}");
    m.insert("japanese_secret_button", "\u{3299}");
    m.insert("japanese_service_charge_button", "\u{1F202}");
    m.insert("japanese_symbol_for_beginner", "\u{1F530}");
    m.insert("japanese_vacancy_button", "\u{1F233}");
    m.insert("jersey", "\u{1F1EF}\u{1F1EA}");
    m.insert("jordan", "\u{1F1EF}\u{1F1F4}");
    m.insert("kazakhstan", "\u{1F1F0}\u{1F1FF}");
    m.insert("kenya", "\u{1F1F0}\u{1F1EA}");
    m.insert("kiribati", "\u{1F1F0}\u{1F1EE}");
    m.insert("kosovo", "\u{1F1FD}\u{1F1F0}");
    m.insert("kuwait", "\u{1F1F0}\u{1F1FC}");
    m.insert("kyrgyzstan", "\u{1F1F0}\u{1F1EC}");
    m.insert("laos", "\u{1F1F1}\u{1F1E6}");
    m.insert("latvia", "\u{1F1F1}\u{1F1FB}");
    m.insert("lebanon", "\u{1F1F1}\u{1F1E7}");
    m.insert("leo", "\u{264C}");
    m.insert("lesotho", "\u{1F1F1}\u{1F1F8}");
    m.insert("liberia", "\u{1F1F1}\u{1F1F7}");
    m.insert("libra", "\u{264E}");
    m.insert("libya", "\u{1F1F1}\u{1F1FE}");
    m.insert("liechtenstein", "\u{1F1F1}\u{1F1EE}");
    m.insert("lithuania", "\u{1F1F1}\u{1F1F9}");
    m.insert("luxembourg", "\u{1F1F1}\u{1F1FA}");
    m.insert("macau_sar_china", "\u{1F1F2}\u{1F1F4}");
    m.insert("macedonia", "\u{1F1F2}\u{1F1F0}");
    m.insert("madagascar", "\u{1F1F2}\u{1F1EC}");
    m.insert("malawi", "\u{1F1F2}\u{1F1FC}");
    m.insert("malaysia", "\u{1F1F2}\u{1F1FE}");
    m.insert("maldives", "\u{1F1F2}\u{1F1FB}");
    m.insert("mali", "\u{1F1F2}\u{1F1F1}");
    m.insert("malta", "\u{1F1F2}\u{1F1F9}");
    m.insert("marshall_islands", "\u{1F1F2}\u{1F1ED}");
    m.insert("martinique", "\u{1F1F2}\u{1F1F6}");
    m.insert("mauritania", "\u{1F1F2}\u{1F1F7}");
    m.insert("mauritius", "\u{1F1F2}\u{1F1FA}");
    m.insert("mayotte", "\u{1F1FE}\u{1F1F9}");
    m.insert("mexico", "\u{1F1F2}\u{1F1FD}");
    m.insert("micronesia", "\u{1F1EB}\u{1F1F2}");
    m.insert("moldova", "\u{1F1F2}\u{1F1E9}");
    m.insert("monaco", "\u{1F1F2}\u{1F1E8}");
    m.insert("mongolia", "\u{1F1F2}\u{1F1F3}");
    m.insert("montenegro", "\u{1F1F2}\u{1F1EA}");
    m.insert("montserrat", "\u{1F1F2}\u{1F1F8}");
    m.insert("morocco", "\u{1F1F2}\u{1F1E6}");
    m.insert("mozambique", "\u{1F1F2}\u{1F1FF}");
    m.insert("mrs._claus", "\u{1F936}");
    m.insert("mrs._claus_dark_skin_tone", "\u{1F936}\u{1F3FF}");
    m.insert("mrs._claus_light_skin_tone", "\u{1F936}\u{1F3FB}");
    m.insert("mrs._claus_medium-dark_skin_tone", "\u{1F936}\u{1F3FE}");
    m.insert("mrs._claus_medium-light_skin_tone", "\u{1F936}\u{1F3FC}");
    m.insert("mrs._claus_medium_skin_tone", "\u{1F936}\u{1F3FD}");
    m.insert("myanmar_(burma)", "\u{1F1F2}\u{1F1F2}");
    m.insert("new_button", "\u{1F195}");
    m.insert("ng_button", "\u{1F196}");
    m.insert("namibia", "\u{1F1F3}\u{1F1E6}");
    m.insert("nauru", "\u{1F1F3}\u{1F1F7}");
    m.insert("nepal", "\u{1F1F3}\u{1F1F5}");
    m.insert("netherlands", "\u{1F1F3}\u{1F1F1}");
    m.insert("new_caledonia", "\u{1F1F3}\u{1F1E8}");
    m.insert("new_zealand", "\u{1F1F3}\u{1F1FF}");
    m.insert("nicaragua", "\u{1F1F3}\u{1F1EE}");
    m.insert("niger", "\u{1F1F3}\u{1F1EA}");
    m.insert("nigeria", "\u{1F1F3}\u{1F1EC}");
    m.insert("niue", "\u{1F1F3}\u{1F1FA}");
    m.insert("norfolk_island", "\u{1F1F3}\u{1F1EB}");
    m.insert("north_korea", "\u{1F1F0}\u{1F1F5}");
    m.insert("northern_mariana_islands", "\u{1F1F2}\u{1F1F5}");
    m.insert("norway", "\u{1F1F3}\u{1F1F4}");
    m.insert("ok_button", "\u{1F197}");
    m.insert("ok_hand", "\u{1F44C}");
    m.insert("ok_hand_dark_skin_tone", "\u{1F44C}\u{1F3FF}");
    m.insert("ok_hand_light_skin_tone", "\u{1F44C}\u{1F3FB}");
    m.insert("ok_hand_medium-dark_skin_tone", "\u{1F44C}\u{1F3FE}");
    m.insert("ok_hand_medium-light_skin_tone", "\u{1F44C}\u{1F3FC}");
    m.insert("ok_hand_medium_skin_tone", "\u{1F44C}\u{1F3FD}");
    m.insert("on!_arrow", "\u{1F51B}");
    m.insert("o_button_(blood_type)", "\u{1F17E}");
    m.insert("oman", "\u{1F1F4}\u{1F1F2}");
    m.insert("ophiuchus", "\u{26CE}");
    m.insert("p_button", "\u{1F17F}");
    m.insert("pakistan", "\u{1F1F5}\u{1F1F0}");
    m.insert("palau", "\u{1F1F5}\u{1F1FC}");
    m.insert("palestinian_territories", "\u{1F1F5}\u{1F1F8}");
    m.insert("panama", "\u{1F1F5}\u{1F1E6}");
    m.insert("papua_new_guinea", "\u{1F1F5}\u{1F1EC}");
    m.insert("paraguay", "\u{1F1F5}\u{1F1FE}");
    m.insert("peru", "\u{1F1F5}\u{1F1EA}");
    m.insert("philippines", "\u{1F1F5}\u{1F1ED}");
    m.insert("pisces", "\u{2653}");
    m.insert("pitcairn_islands", "\u{1F1F5}\u{1F1F3}");
    m.insert("poland", "\u{1F1F5}\u{1F1F1}");
    m.insert("portugal", "\u{1F1F5}\u{1F1F9}");
    m.insert("puerto_rico", "\u{1F1F5}\u{1F1F7}");
    m.insert("qatar", "\u{1F1F6}\u{1F1E6}");
    m.insert("romania", "\u{1F1F7}\u{1F1F4}");
    m.insert("russia", "\u{1F1F7}\u{1F1FA}");
    m.insert("rwanda", "\u{1F1F7}\u{1F1FC}");
    m.insert("réunion", "\u{1F1F7}\u{1F1EA}");
    m.insert("soon_arrow", "\u{1F51C}");
    m.insert("sos_button", "\u{1F198}");
    m.insert("sagittarius", "\u{2650}");
    m.insert("samoa", "\u{1F1FC}\u{1F1F8}");
    m.insert("san_marino", "\u{1F1F8}\u{1F1F2}");
    m.insert("santa_claus", "\u{1F385}");
    m.insert("santa_claus_dark_skin_tone", "\u{1F385}\u{1F3FF}");
    m.insert("santa_claus_light_skin_tone", "\u{1F385}\u{1F3FB}");
    m.insert("santa_claus_medium-dark_skin_tone", "\u{1F385}\u{1F3FE}");
    m.insert("santa_claus_medium-light_skin_tone", "\u{1F385}\u{1F3FC}");
    m.insert("santa_claus_medium_skin_tone", "\u{1F385}\u{1F3FD}");
    m.insert("saudi_arabia", "\u{1F1F8}\u{1F1E6}");
    m.insert("scorpio", "\u{264F}");
    m.insert(
        "scotland",
        "\u{1F3F4}\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}",
    );
    m.insert("senegal", "\u{1F1F8}\u{1F1F3}");
    m.insert("serbia", "\u{1F1F7}\u{1F1F8}");
    m.insert("seychelles", "\u{1F1F8}\u{1F1E8}");
    m.insert("sierra_leone", "\u{1F1F8}\u{1F1F1}");
    m.insert("singapore", "\u{1F1F8}\u{1F1EC}");
    m.insert("sint_maarten", "\u{1F1F8}\u{1F1FD}");
    m.insert("slovakia", "\u{1F1F8}\u{1F1F0}");
    m.insert("slovenia", "\u{1F1F8}\u{1F1EE}");
    m.insert("solomon_islands", "\u{1F1F8}\u{1F1E7}");
    m.insert("somalia", "\u{1F1F8}\u{1F1F4}");
    m.insert("south_africa", "\u{1F1FF}\u{1F1E6}");
    m.insert(
        "south_georgia_&_south_sandwich_islands",
        "\u{1F1EC}\u{1F1F8}",
    );
    m.insert("south_korea", "\u{1F1F0}\u{1F1F7}");
    m.insert("south_sudan", "\u{1F1F8}\u{1F1F8}");
    m.insert("spain", "\u{1F1EA}\u{1F1F8}");
    m.insert("sri_lanka", "\u{1F1F1}\u{1F1F0}");
    m.insert("st._barthélemy", "\u{1F1E7}\u{1F1F1}");
    m.insert("st._helena", "\u{1F1F8}\u{1F1ED}");
    m.insert("st._kitts_&_nevis", "\u{1F1F0}\u{1F1F3}");
    m.insert("st._lucia", "\u{1F1F1}\u{1F1E8}");
    m.insert("st._martin", "\u{1F1F2}\u{1F1EB}");
    m.insert("st._pierre_&_miquelon", "\u{1F1F5}\u{1F1F2}");
    m.insert("st._vincent_&_grenadines", "\u{1F1FB}\u{1F1E8}");
    m.insert("statue_of_liberty", "\u{1F5FD}");
    m.insert("sudan", "\u{1F1F8}\u{1F1E9}");
    m.insert("suriname", "\u{1F1F8}\u{1F1F7}");
    m.insert("svalbard_&_jan_mayen", "\u{1F1F8}\u{1F1EF}");
    m.insert("swaziland", "\u{1F1F8}\u{1F1FF}");
    m.insert("sweden", "\u{1F1F8}\u{1F1EA}");
    m.insert("switzerland", "\u{1F1E8}\u{1F1ED}");
    m.insert("syria", "\u{1F1F8}\u{1F1FE}");
    m.insert("são_tomé_&_príncipe", "\u{1F1F8}\u{1F1F9}");
    m.insert("t-rex", "\u{1F996}");
    m.insert("top_arrow", "\u{1F51D}");
    m.insert("taiwan", "\u{1F1F9}\u{1F1FC}");
    m.insert("tajikistan", "\u{1F1F9}\u{1F1EF}");
    m.insert("tanzania", "\u{1F1F9}\u{1F1FF}");
    m.insert("taurus", "\u{2649}");
    m.insert("thailand", "\u{1F1F9}\u{1F1ED}");
    m.insert("timor-leste", "\u{1F1F9}\u{1F1F1}");
    m.insert("togo", "\u{1F1F9}\u{1F1EC}");
    m.insert("tokelau", "\u{1F1F9}\u{1F1F0}");
    m.insert("tokyo_tower", "\u{1F5FC}");
    m.insert("tonga", "\u{1F1F9}\u{1F1F4}");
    m.insert("trinidad_&_tobago", "\u{1F1F9}\u{1F1F9}");
    m.insert("tristan_da_cunha", "\u{1F1F9}\u{1F1E6}");
    m.insert("tunisia", "\u{1F1F9}\u{1F1F3}");
    m.insert("turkey", "\u{1F983}");
    m.insert("turkmenistan", "\u{1F1F9}\u{1F1F2}");
    m.insert("turks_&_caicos_islands", "\u{1F1F9}\u{1F1E8}");
    m.insert("tuvalu", "\u{1F1F9}\u{1F1FB}");
    m.insert("u.s._outlying_islands", "\u{1F1FA}\u{1F1F2}");
    m.insert("u.s._virgin_islands", "\u{1F1FB}\u{1F1EE}");
    m.insert("up!_button", "\u{1F199}");
    m.insert("uganda", "\u{1F1FA}\u{1F1EC}");
    m.insert("ukraine", "\u{1F1FA}\u{1F1E6}");
    m.insert("united_arab_emirates", "\u{1F1E6}\u{1F1EA}");
    m.insert("united_kingdom", "\u{1F1EC}\u{1F1E7}");
    m.insert("united_nations", "\u{1F1FA}\u{1F1F3}");
    m.insert("united_states", "\u{1F1FA}\u{1F1F8}");
    m.insert("uruguay", "\u{1F1FA}\u{1F1FE}");
    m.insert("uzbekistan", "\u{1F1FA}\u{1F1FF}");
    m.insert("vs_button", "\u{1F19A}");
    m.insert("vanuatu", "\u{1F1FB}\u{1F1FA}");
    m.insert("vatican_city", "\u{1F1FB}\u{1F1E6}");
    m.insert("venezuela", "\u{1F1FB}\u{1F1EA}");
    m.insert("vietnam", "\u{1F1FB}\u{1F1F3}");
    m.insert("virgo", "\u{264D}");
    m.insert(
        "wales",
        "\u{1F3F4}\u{E0067}\u{E0062}\u{E0077}\u{E006C}\u{E0073}\u{E007F}",
    );
    m.insert("wallis_&_futuna", "\u{1F1FC}\u{1F1EB}");
    m.insert("western_sahara", "\u{1F1EA}\u{1F1ED}");
    m.insert("yemen", "\u{1F1FE}\u{1F1EA}");
    m.insert("zambia", "\u{1F1FF}\u{1F1F2}");
    m.insert("zimbabwe", "\u{1F1FF}\u{1F1FC}");
    m.insert("abacus", "\u{1F9EE}");
    m.insert("adhesive_bandage", "\u{1FA79}");
    m.insert("admission_tickets", "\u{1F39F}");
    m.insert("adult", "\u{1F9D1}");
    m.insert("adult_dark_skin_tone", "\u{1F9D1}\u{1F3FF}");
    m.insert("adult_light_skin_tone", "\u{1F9D1}\u{1F3FB}");
    m.insert("adult_medium-dark_skin_tone", "\u{1F9D1}\u{1F3FE}");
    m.insert("adult_medium-light_skin_tone", "\u{1F9D1}\u{1F3FC}");
    m.insert("adult_medium_skin_tone", "\u{1F9D1}\u{1F3FD}");
    m.insert("aerial_tramway", "\u{1F6A1}");
    m.insert("airplane", "\u{2708}");
    m.insert("airplane_arrival", "\u{1F6EC}");
    m.insert("airplane_departure", "\u{1F6EB}");
    m.insert("alarm_clock", "\u{23F0}");
    m.insert("alembic", "\u{2697}");
    m.insert("alien", "\u{1F47D}");
    m.insert("alien_monster", "\u{1F47E}");
    m.insert("ambulance", "\u{1F691}");
    m.insert("american_football", "\u{1F3C8}");
    m.insert("amphora", "\u{1F3FA}");
    m.insert("anchor", "\u{2693}");
    m.insert("anger_symbol", "\u{1F4A2}");
    m.insert("angry_face", "\u{1F620}");
    m.insert("angry_face_with_horns", "\u{1F47F}");
    m.insert("anguished_face", "\u{1F627}");
    m.insert("ant", "\u{1F41C}");
    m.insert("antenna_bars", "\u{1F4F6}");
    m.insert("anxious_face_with_sweat", "\u{1F630}");
    m.insert("articulated_lorry", "\u{1F69B}");
    m.insert("artist_palette", "\u{1F3A8}");
    m.insert("astonished_face", "\u{1F632}");
    m.insert("atom_symbol", "\u{269B}");
    m.insert("auto_rickshaw", "\u{1F6FA}");
    m.insert("automobile", "\u{1F697}");
    m.insert("avocado", "\u{1F951}");
    m.insert("axe", "\u{1FA93}");
    m.insert("baby", "\u{1F476}");
    m.insert("baby_angel", "\u{1F47C}");
    m.insert("baby_angel_dark_skin_tone", "\u{1F47C}\u{1F3FF}");
    m.insert("baby_angel_light_skin_tone", "\u{1F47C}\u{1F3FB}");
    m.insert("baby_angel_medium-dark_skin_tone", "\u{1F47C}\u{1F3FE}");
    m.insert("baby_angel_medium-light_skin_tone", "\u{1F47C}\u{1F3FC}");
    m.insert("baby_angel_medium_skin_tone", "\u{1F47C}\u{1F3FD}");
    m.insert("baby_bottle", "\u{1F37C}");
    m.insert("baby_chick", "\u{1F424}");
    m.insert("baby_dark_skin_tone", "\u{1F476}\u{1F3FF}");
    m.insert("baby_light_skin_tone", "\u{1F476}\u{1F3FB}");
    m.insert("baby_medium-dark_skin_tone", "\u{1F476}\u{1F3FE}");
    m.insert("baby_medium-light_skin_tone", "\u{1F476}\u{1F3FC}");
    m.insert("baby_medium_skin_tone", "\u{1F476}\u{1F3FD}");
    m.insert("baby_symbol", "\u{1F6BC}");
    m.insert("backhand_index_pointing_down", "\u{1F447}");
    m.insert(
        "backhand_index_pointing_down_dark_skin_tone",
        "\u{1F447}\u{1F3FF}",
    );
    m.insert(
        "backhand_index_pointing_down_light_skin_tone",
        "\u{1F447}\u{1F3FB}",
    );
    m.insert(
        "backhand_index_pointing_down_medium-dark_skin_tone",
        "\u{1F447}\u{1F3FE}",
    );
    m.insert(
        "backhand_index_pointing_down_medium-light_skin_tone",
        "\u{1F447}\u{1F3FC}",
    );
    m.insert(
        "backhand_index_pointing_down_medium_skin_tone",
        "\u{1F447}\u{1F3FD}",
    );
    m.insert("backhand_index_pointing_left", "\u{1F448}");
    m.insert(
        "backhand_index_pointing_left_dark_skin_tone",
        "\u{1F448}\u{1F3FF}",
    );
    m.insert(
        "backhand_index_pointing_left_light_skin_tone",
        "\u{1F448}\u{1F3FB}",
    );
    m.insert(
        "backhand_index_pointing_left_medium-dark_skin_tone",
        "\u{1F448}\u{1F3FE}",
    );
    m.insert(
        "backhand_index_pointing_left_medium-light_skin_tone",
        "\u{1F448}\u{1F3FC}",
    );
    m.insert(
        "backhand_index_pointing_left_medium_skin_tone",
        "\u{1F448}\u{1F3FD}",
    );
    m.insert("backhand_index_pointing_right", "\u{1F449}");
    m.insert(
        "backhand_index_pointing_right_dark_skin_tone",
        "\u{1F449}\u{1F3FF}",
    );
    m.insert(
        "backhand_index_pointing_right_light_skin_tone",
        "\u{1F449}\u{1F3FB}",
    );
    m.insert(
        "backhand_index_pointing_right_medium-dark_skin_tone",
        "\u{1F449}\u{1F3FE}",
    );
    m.insert(
        "backhand_index_pointing_right_medium-light_skin_tone",
        "\u{1F449}\u{1F3FC}",
    );
    m.insert(
        "backhand_index_pointing_right_medium_skin_tone",
        "\u{1F449}\u{1F3FD}",
    );
    m.insert("backhand_index_pointing_up", "\u{1F446}");
    m.insert(
        "backhand_index_pointing_up_dark_skin_tone",
        "\u{1F446}\u{1F3FF}",
    );
    m.insert(
        "backhand_index_pointing_up_light_skin_tone",
        "\u{1F446}\u{1F3FB}",
    );
    m.insert(
        "backhand_index_pointing_up_medium-dark_skin_tone",
        "\u{1F446}\u{1F3FE}",
    );
    m.insert(
        "backhand_index_pointing_up_medium-light_skin_tone",
        "\u{1F446}\u{1F3FC}",
    );
    m.insert(
        "backhand_index_pointing_up_medium_skin_tone",
        "\u{1F446}\u{1F3FD}",
    );
    m.insert("bacon", "\u{1F953}");
    m.insert("badger", "\u{1F9A1}");
    m.insert("badminton", "\u{1F3F8}");
    m.insert("bagel", "\u{1F96F}");
    m.insert("baggage_claim", "\u{1F6C4}");
    m.insert("baguette_bread", "\u{1F956}");
    m.insert("balance_scale", "\u{2696}");
    m.insert("bald", "\u{1F9B2}");
    m.insert("bald_man", "\u{1F468}\u{200D}\u{1F9B2}");
    m.insert("bald_woman", "\u{1F469}\u{200D}\u{1F9B2}");
    m.insert("ballet_shoes", "\u{1FA70}");
    m.insert("balloon", "\u{1F388}");
    m.insert("ballot_box_with_ballot", "\u{1F5F3}");
    m.insert("ballot_box_with_check", "\u{2611}");
    m.insert("banana", "\u{1F34C}");
    m.insert("banjo", "\u{1FA95}");
    m.insert("bank", "\u{1F3E6}");
    m.insert("bar_chart", "\u{1F4CA}");
    m.insert("barber_pole", "\u{1F488}");
    m.insert("baseball", "\u{26BE}");
    m.insert("basket", "\u{1F9FA}");
    m.insert("basketball", "\u{1F3C0}");
    m.insert("bat", "\u{1F987}");
    m.insert("bathtub", "\u{1F6C1}");
    m.insert("battery", "\u{1F50B}");
    m.insert("beach_with_umbrella", "\u{1F3D6}");
    m.insert("beaming_face_with_smiling_eyes", "\u{1F601}");
    m.insert("bear_face", "\u{1F43B}");
    m.insert("bearded_person", "\u{1F9D4}");
    m.insert("bearded_person_dark_skin_tone", "\u{1F9D4}\u{1F3FF}");
    m.insert("bearded_person_light_skin_tone", "\u{1F9D4}\u{1F3FB}");
    m.insert("bearded_person_medium-dark_skin_tone", "\u{1F9D4}\u{1F3FE}");
    m.insert(
        "bearded_person_medium-light_skin_tone",
        "\u{1F9D4}\u{1F3FC}",
    );
    m.insert("bearded_person_medium_skin_tone", "\u{1F9D4}\u{1F3FD}");
    m.insert("beating_heart", "\u{1F493}");
    m.insert("bed", "\u{1F6CF}");
    m.insert("beer_mug", "\u{1F37A}");
    m.insert("bell", "\u{1F514}");
    m.insert("bell_with_slash", "\u{1F515}");
    m.insert("bellhop_bell", "\u{1F6CE}");
    m.insert("bento_box", "\u{1F371}");
    m.insert("beverage_box", "\u{1F9C3}");
    m.insert("bicycle", "\u{1F6B2}");
    m.insert("bikini", "\u{1F459}");
    m.insert("billed_cap", "\u{1F9E2}");
    m.insert("biohazard", "\u{2623}");
    m.insert("bird", "\u{1F426}");
    m.insert("birthday_cake", "\u{1F382}");
    m.insert("black_circle", "\u{26AB}");
    m.insert("black_flag", "\u{1F3F4}");
    m.insert("black_heart", "\u{1F5A4}");
    m.insert("black_large_square", "\u{2B1B}");
    m.insert("black_medium-small_square", "\u{25FE}");
    m.insert("black_medium_square", "\u{25FC}");
    m.insert("black_nib", "\u{2712}");
    m.insert("black_small_square", "\u{25AA}");
    m.insert("black_square_button", "\u{1F532}");
    m.insert("blond-haired_man", "\u{1F471}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "blond-haired_man_dark_skin_tone",
        "\u{1F471}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "blond-haired_man_light_skin_tone",
        "\u{1F471}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "blond-haired_man_medium-dark_skin_tone",
        "\u{1F471}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "blond-haired_man_medium-light_skin_tone",
        "\u{1F471}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "blond-haired_man_medium_skin_tone",
        "\u{1F471}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("blond-haired_person", "\u{1F471}");
    m.insert("blond-haired_person_dark_skin_tone", "\u{1F471}\u{1F3FF}");
    m.insert("blond-haired_person_light_skin_tone", "\u{1F471}\u{1F3FB}");
    m.insert(
        "blond-haired_person_medium-dark_skin_tone",
        "\u{1F471}\u{1F3FE}",
    );
    m.insert(
        "blond-haired_person_medium-light_skin_tone",
        "\u{1F471}\u{1F3FC}",
    );
    m.insert("blond-haired_person_medium_skin_tone", "\u{1F471}\u{1F3FD}");
    m.insert("blond-haired_woman", "\u{1F471}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "blond-haired_woman_dark_skin_tone",
        "\u{1F471}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "blond-haired_woman_light_skin_tone",
        "\u{1F471}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "blond-haired_woman_medium-dark_skin_tone",
        "\u{1F471}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "blond-haired_woman_medium-light_skin_tone",
        "\u{1F471}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "blond-haired_woman_medium_skin_tone",
        "\u{1F471}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("blossom", "\u{1F33C}");
    m.insert("blowfish", "\u{1F421}");
    m.insert("blue_book", "\u{1F4D8}");
    m.insert("blue_circle", "\u{1F535}");
    m.insert("blue_heart", "\u{1F499}");
    m.insert("blue_square", "\u{1F7E6}");
    m.insert("boar", "\u{1F417}");
    m.insert("bomb", "\u{1F4A3}");
    m.insert("bone", "\u{1F9B4}");
    m.insert("bookmark", "\u{1F516}");
    m.insert("bookmark_tabs", "\u{1F4D1}");
    m.insert("books", "\u{1F4DA}");
    m.insert("bottle_with_popping_cork", "\u{1F37E}");
    m.insert("bouquet", "\u{1F490}");
    m.insert("bow_and_arrow", "\u{1F3F9}");
    m.insert("bowl_with_spoon", "\u{1F963}");
    m.insert("bowling", "\u{1F3B3}");
    m.insert("boxing_glove", "\u{1F94A}");
    m.insert("boy", "\u{1F466}");
    m.insert("boy_dark_skin_tone", "\u{1F466}\u{1F3FF}");
    m.insert("boy_light_skin_tone", "\u{1F466}\u{1F3FB}");
    m.insert("boy_medium-dark_skin_tone", "\u{1F466}\u{1F3FE}");
    m.insert("boy_medium-light_skin_tone", "\u{1F466}\u{1F3FC}");
    m.insert("boy_medium_skin_tone", "\u{1F466}\u{1F3FD}");
    m.insert("brain", "\u{1F9E0}");
    m.insert("bread", "\u{1F35E}");
    m.insert("breast-feeding", "\u{1F931}");
    m.insert("breast-feeding_dark_skin_tone", "\u{1F931}\u{1F3FF}");
    m.insert("breast-feeding_light_skin_tone", "\u{1F931}\u{1F3FB}");
    m.insert("breast-feeding_medium-dark_skin_tone", "\u{1F931}\u{1F3FE}");
    m.insert(
        "breast-feeding_medium-light_skin_tone",
        "\u{1F931}\u{1F3FC}",
    );
    m.insert("breast-feeding_medium_skin_tone", "\u{1F931}\u{1F3FD}");
    m.insert("brick", "\u{1F9F1}");
    m.insert("bride_with_veil", "\u{1F470}");
    m.insert("bride_with_veil_dark_skin_tone", "\u{1F470}\u{1F3FF}");
    m.insert("bride_with_veil_light_skin_tone", "\u{1F470}\u{1F3FB}");
    m.insert(
        "bride_with_veil_medium-dark_skin_tone",
        "\u{1F470}\u{1F3FE}",
    );
    m.insert(
        "bride_with_veil_medium-light_skin_tone",
        "\u{1F470}\u{1F3FC}",
    );
    m.insert("bride_with_veil_medium_skin_tone", "\u{1F470}\u{1F3FD}");
    m.insert("bridge_at_night", "\u{1F309}");
    m.insert("briefcase", "\u{1F4BC}");
    m.insert("briefs", "\u{1FA72}");
    m.insert("bright_button", "\u{1F506}");
    m.insert("broccoli", "\u{1F966}");
    m.insert("broken_heart", "\u{1F494}");
    m.insert("broom", "\u{1F9F9}");
    m.insert("brown_circle", "\u{1F7E4}");
    m.insert("brown_heart", "\u{1F90E}");
    m.insert("brown_square", "\u{1F7EB}");
    m.insert("bug", "\u{1F41B}");
    m.insert("building_construction", "\u{1F3D7}");
    m.insert("bullet_train", "\u{1F685}");
    m.insert("burrito", "\u{1F32F}");
    m.insert("bus", "\u{1F68C}");
    m.insert("bus_stop", "\u{1F68F}");
    m.insert("bust_in_silhouette", "\u{1F464}");
    m.insert("busts_in_silhouette", "\u{1F465}");
    m.insert("butter", "\u{1F9C8}");
    m.insert("butterfly", "\u{1F98B}");
    m.insert("cactus", "\u{1F335}");
    m.insert("calendar", "\u{1F4C6}");
    m.insert("call_me_hand", "\u{1F919}");
    m.insert("call_me_hand_dark_skin_tone", "\u{1F919}\u{1F3FF}");
    m.insert("call_me_hand_light_skin_tone", "\u{1F919}\u{1F3FB}");
    m.insert("call_me_hand_medium-dark_skin_tone", "\u{1F919}\u{1F3FE}");
    m.insert("call_me_hand_medium-light_skin_tone", "\u{1F919}\u{1F3FC}");
    m.insert("call_me_hand_medium_skin_tone", "\u{1F919}\u{1F3FD}");
    m.insert("camel", "\u{1F42B}");
    m.insert("camera", "\u{1F4F7}");
    m.insert("camera_with_flash", "\u{1F4F8}");
    m.insert("camping", "\u{1F3D5}");
    m.insert("candle", "\u{1F56F}");
    m.insert("candy", "\u{1F36C}");
    m.insert("canned_food", "\u{1F96B}");
    m.insert("canoe", "\u{1F6F6}");
    m.insert("card_file_box", "\u{1F5C3}");
    m.insert("card_index", "\u{1F4C7}");
    m.insert("card_index_dividers", "\u{1F5C2}");
    m.insert("carousel_horse", "\u{1F3A0}");
    m.insert("carp_streamer", "\u{1F38F}");
    m.insert("carrot", "\u{1F955}");
    m.insert("castle", "\u{1F3F0}");
    m.insert("cat", "\u{1F431}");
    m.insert("cat_face", "\u{1F431}");
    m.insert("cat_face_with_tears_of_joy", "\u{1F639}");
    m.insert("cat_face_with_wry_smile", "\u{1F63C}");
    m.insert("chains", "\u{26D3}");
    m.insert("chair", "\u{1FA91}");
    m.insert("chart_decreasing", "\u{1F4C9}");
    m.insert("chart_increasing", "\u{1F4C8}");
    m.insert("chart_increasing_with_yen", "\u{1F4B9}");
    m.insert("cheese_wedge", "\u{1F9C0}");
    m.insert("chequered_flag", "\u{1F3C1}");
    m.insert("cherries", "\u{1F352}");
    m.insert("cherry_blossom", "\u{1F338}");
    m.insert("chess_pawn", "\u{265F}");
    m.insert("chestnut", "\u{1F330}");
    m.insert("chicken", "\u{1F414}");
    m.insert("child", "\u{1F9D2}");
    m.insert("child_dark_skin_tone", "\u{1F9D2}\u{1F3FF}");
    m.insert("child_light_skin_tone", "\u{1F9D2}\u{1F3FB}");
    m.insert("child_medium-dark_skin_tone", "\u{1F9D2}\u{1F3FE}");
    m.insert("child_medium-light_skin_tone", "\u{1F9D2}\u{1F3FC}");
    m.insert("child_medium_skin_tone", "\u{1F9D2}\u{1F3FD}");
    m.insert("children_crossing", "\u{1F6B8}");
    m.insert("chipmunk", "\u{1F43F}");
    m.insert("chocolate_bar", "\u{1F36B}");
    m.insert("chopsticks", "\u{1F962}");
    m.insert("church", "\u{26EA}");
    m.insert("cigarette", "\u{1F6AC}");
    m.insert("cinema", "\u{1F3A6}");
    m.insert("circled_m", "\u{24C2}");
    m.insert("circus_tent", "\u{1F3AA}");
    m.insert("cityscape", "\u{1F3D9}");
    m.insert("cityscape_at_dusk", "\u{1F306}");
    m.insert("clamp", "\u{1F5DC}");
    m.insert("clapper_board", "\u{1F3AC}");
    m.insert("clapping_hands", "\u{1F44F}");
    m.insert("clapping_hands_dark_skin_tone", "\u{1F44F}\u{1F3FF}");
    m.insert("clapping_hands_light_skin_tone", "\u{1F44F}\u{1F3FB}");
    m.insert("clapping_hands_medium-dark_skin_tone", "\u{1F44F}\u{1F3FE}");
    m.insert(
        "clapping_hands_medium-light_skin_tone",
        "\u{1F44F}\u{1F3FC}",
    );
    m.insert("clapping_hands_medium_skin_tone", "\u{1F44F}\u{1F3FD}");
    m.insert("classical_building", "\u{1F3DB}");
    m.insert("clinking_beer_mugs", "\u{1F37B}");
    m.insert("clinking_glasses", "\u{1F942}");
    m.insert("clipboard", "\u{1F4CB}");
    m.insert("clockwise_vertical_arrows", "\u{1F503}");
    m.insert("closed_book", "\u{1F4D5}");
    m.insert("closed_mailbox_with_lowered_flag", "\u{1F4EA}");
    m.insert("closed_mailbox_with_raised_flag", "\u{1F4EB}");
    m.insert("closed_umbrella", "\u{1F302}");
    m.insert("cloud", "\u{2601}");
    m.insert("cloud_with_lightning", "\u{1F329}");
    m.insert("cloud_with_lightning_and_rain", "\u{26C8}");
    m.insert("cloud_with_rain", "\u{1F327}");
    m.insert("cloud_with_snow", "\u{1F328}");
    m.insert("clown_face", "\u{1F921}");
    m.insert("club_suit", "\u{2663}");
    m.insert("clutch_bag", "\u{1F45D}");
    m.insert("coat", "\u{1F9E5}");
    m.insert("cocktail_glass", "\u{1F378}");
    m.insert("coconut", "\u{1F965}");
    m.insert("coffin", "\u{26B0}");
    m.insert("cold_face", "\u{1F976}");
    m.insert("collision", "\u{1F4A5}");
    m.insert("comet", "\u{2604}");
    m.insert("compass", "\u{1F9ED}");
    m.insert("computer_disk", "\u{1F4BD}");
    m.insert("computer_mouse", "\u{1F5B1}");
    m.insert("confetti_ball", "\u{1F38A}");
    m.insert("confounded_face", "\u{1F616}");
    m.insert("confused_face", "\u{1F615}");
    m.insert("construction", "\u{1F6A7}");
    m.insert("construction_worker", "\u{1F477}");
    m.insert("construction_worker_dark_skin_tone", "\u{1F477}\u{1F3FF}");
    m.insert("construction_worker_light_skin_tone", "\u{1F477}\u{1F3FB}");
    m.insert(
        "construction_worker_medium-dark_skin_tone",
        "\u{1F477}\u{1F3FE}",
    );
    m.insert(
        "construction_worker_medium-light_skin_tone",
        "\u{1F477}\u{1F3FC}",
    );
    m.insert("construction_worker_medium_skin_tone", "\u{1F477}\u{1F3FD}");
    m.insert("control_knobs", "\u{1F39B}");
    m.insert("convenience_store", "\u{1F3EA}");
    m.insert("cooked_rice", "\u{1F35A}");
    m.insert("cookie", "\u{1F36A}");
    m.insert("cooking", "\u{1F373}");
    m.insert("copyright", "\u{A9}");
    m.insert("couch_and_lamp", "\u{1F6CB}");
    m.insert("counterclockwise_arrows_button", "\u{1F504}");
    m.insert("couple_with_heart", "\u{1F491}");
    m.insert(
        "couple_with_heart_man_man",
        "\u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468}",
    );
    m.insert(
        "couple_with_heart_woman_man",
        "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468}",
    );
    m.insert(
        "couple_with_heart_woman_woman",
        "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F469}",
    );
    m.insert("cow", "\u{1F42E}");
    m.insert("cow_face", "\u{1F42E}");
    m.insert("cowboy_hat_face", "\u{1F920}");
    m.insert("crab", "\u{1F980}");
    m.insert("crayon", "\u{1F58D}");
    m.insert("credit_card", "\u{1F4B3}");
    m.insert("crescent_moon", "\u{1F319}");
    m.insert("cricket", "\u{1F997}");
    m.insert("cricket_game", "\u{1F3CF}");
    m.insert("crocodile", "\u{1F40A}");
    m.insert("croissant", "\u{1F950}");
    m.insert("cross_mark", "\u{274C}");
    m.insert("cross_mark_button", "\u{274E}");
    m.insert("crossed_fingers", "\u{1F91E}");
    m.insert("crossed_fingers_dark_skin_tone", "\u{1F91E}\u{1F3FF}");
    m.insert("crossed_fingers_light_skin_tone", "\u{1F91E}\u{1F3FB}");
    m.insert(
        "crossed_fingers_medium-dark_skin_tone",
        "\u{1F91E}\u{1F3FE}",
    );
    m.insert(
        "crossed_fingers_medium-light_skin_tone",
        "\u{1F91E}\u{1F3FC}",
    );
    m.insert("crossed_fingers_medium_skin_tone", "\u{1F91E}\u{1F3FD}");
    m.insert("crossed_flags", "\u{1F38C}");
    m.insert("crossed_swords", "\u{2694}");
    m.insert("crown", "\u{1F451}");
    m.insert("crying_cat_face", "\u{1F63F}");
    m.insert("crying_face", "\u{1F622}");
    m.insert("crystal_ball", "\u{1F52E}");
    m.insert("cucumber", "\u{1F952}");
    m.insert("cupcake", "\u{1F9C1}");
    m.insert("cup_with_straw", "\u{1F964}");
    m.insert("curling_stone", "\u{1F94C}");
    m.insert("curly_hair", "\u{1F9B1}");
    m.insert("curly-haired_man", "\u{1F468}\u{200D}\u{1F9B1}");
    m.insert("curly-haired_woman", "\u{1F469}\u{200D}\u{1F9B1}");
    m.insert("curly_loop", "\u{27B0}");
    m.insert("currency_exchange", "\u{1F4B1}");
    m.insert("curry_rice", "\u{1F35B}");
    m.insert("custard", "\u{1F36E}");
    m.insert("customs", "\u{1F6C3}");
    m.insert("cut_of_meat", "\u{1F969}");
    m.insert("cyclone", "\u{1F300}");
    m.insert("dagger", "\u{1F5E1}");
    m.insert("dango", "\u{1F361}");
    m.insert("dashing_away", "\u{1F4A8}");
    m.insert("deaf_person", "\u{1F9CF}");
    m.insert("deciduous_tree", "\u{1F333}");
    m.insert("deer", "\u{1F98C}");
    m.insert("delivery_truck", "\u{1F69A}");
    m.insert("department_store", "\u{1F3EC}");
    m.insert("derelict_house", "\u{1F3DA}");
    m.insert("desert", "\u{1F3DC}");
    m.insert("desert_island", "\u{1F3DD}");
    m.insert("desktop_computer", "\u{1F5A5}");
    m.insert("detective", "\u{1F575}");
    m.insert("detective_dark_skin_tone", "\u{1F575}\u{1F3FF}");
    m.insert("detective_light_skin_tone", "\u{1F575}\u{1F3FB}");
    m.insert("detective_medium-dark_skin_tone", "\u{1F575}\u{1F3FE}");
    m.insert("detective_medium-light_skin_tone", "\u{1F575}\u{1F3FC}");
    m.insert("detective_medium_skin_tone", "\u{1F575}\u{1F3FD}");
    m.insert("diamond_suit", "\u{2666}");
    m.insert("diamond_with_a_dot", "\u{1F4A0}");
    m.insert("dim_button", "\u{1F505}");
    m.insert("direct_hit", "\u{1F3AF}");
    m.insert("disappointed_face", "\u{1F61E}");
    m.insert("diving_mask", "\u{1F93F}");
    m.insert("diya_lamp", "\u{1FA94}");
    m.insert("dizzy", "\u{1F4AB}");
    m.insert("dizzy_face", "\u{1F635}");
    m.insert("dna", "\u{1F9EC}");
    m.insert("dog", "\u{1F436}");
    m.insert("dog_face", "\u{1F436}");
    m.insert("dollar_banknote", "\u{1F4B5}");
    m.insert("dolphin", "\u{1F42C}");
    m.insert("door", "\u{1F6AA}");
    m.insert("dotted_six-pointed_star", "\u{1F52F}");
    m.insert("double_curly_loop", "\u{27BF}");
    m.insert("double_exclamation_mark", "\u{203C}");
    m.insert("doughnut", "\u{1F369}");
    m.insert("dove", "\u{1F54A}");
    m.insert("down-left_arrow", "\u{2199}");
    m.insert("down-right_arrow", "\u{2198}");
    m.insert("down_arrow", "\u{2B07}");
    m.insert("downcast_face_with_sweat", "\u{1F613}");
    m.insert("downwards_button", "\u{1F53D}");
    m.insert("dragon", "\u{1F409}");
    m.insert("dragon_face", "\u{1F432}");
    m.insert("dress", "\u{1F457}");
    m.insert("drooling_face", "\u{1F924}");
    m.insert("drop_of_blood", "\u{1FA78}");
    m.insert("droplet", "\u{1F4A7}");
    m.insert("drum", "\u{1F941}");
    m.insert("duck", "\u{1F986}");
    m.insert("dumpling", "\u{1F95F}");
    m.insert("dvd", "\u{1F4C0}");
    m.insert("e-mail", "\u{1F4E7}");
    m.insert("eagle", "\u{1F985}");
    m.insert("ear", "\u{1F442}");
    m.insert("ear_dark_skin_tone", "\u{1F442}\u{1F3FF}");
    m.insert("ear_light_skin_tone", "\u{1F442}\u{1F3FB}");
    m.insert("ear_medium-dark_skin_tone", "\u{1F442}\u{1F3FE}");
    m.insert("ear_medium-light_skin_tone", "\u{1F442}\u{1F3FC}");
    m.insert("ear_medium_skin_tone", "\u{1F442}\u{1F3FD}");
    m.insert("ear_of_corn", "\u{1F33D}");
    m.insert("ear_with_hearing_aid", "\u{1F9BB}");
    m.insert("egg", "\u{1F373}");
    m.insert("eggplant", "\u{1F346}");
    m.insert("eight-pointed_star", "\u{2734}");
    m.insert("eight-spoked_asterisk", "\u{2733}");
    m.insert("eight-thirty", "\u{1F563}");
    m.insert("eight_o’clock", "\u{1F557}");
    m.insert("eject_button", "\u{23CF}");
    m.insert("electric_plug", "\u{1F50C}");
    m.insert("elephant", "\u{1F418}");
    m.insert("eleven-thirty", "\u{1F566}");
    m.insert("eleven_o’clock", "\u{1F55A}");
    m.insert("elf", "\u{1F9DD}");
    m.insert("elf_dark_skin_tone", "\u{1F9DD}\u{1F3FF}");
    m.insert("elf_light_skin_tone", "\u{1F9DD}\u{1F3FB}");
    m.insert("elf_medium-dark_skin_tone", "\u{1F9DD}\u{1F3FE}");
    m.insert("elf_medium-light_skin_tone", "\u{1F9DD}\u{1F3FC}");
    m.insert("elf_medium_skin_tone", "\u{1F9DD}\u{1F3FD}");
    m.insert("envelope", "\u{2709}");
    m.insert("envelope_with_arrow", "\u{1F4E9}");
    m.insert("euro_banknote", "\u{1F4B6}");
    m.insert("evergreen_tree", "\u{1F332}");
    m.insert("ewe", "\u{1F411}");
    m.insert("exclamation_mark", "\u{2757}");
    m.insert("exclamation_question_mark", "\u{2049}");
    m.insert("exploding_head", "\u{1F92F}");
    m.insert("expressionless_face", "\u{1F611}");
    m.insert("eye", "\u{1F441}");
    m.insert(
        "eye_in_speech_bubble",
        "\u{1F441}\u{FE0F}\u{200D}\u{1F5E8}\u{FE0F}",
    );
    m.insert("eyes", "\u{1F440}");
    m.insert("face_blowing_a_kiss", "\u{1F618}");
    m.insert("face_savoring_food", "\u{1F60B}");
    m.insert("face_screaming_in_fear", "\u{1F631}");
    m.insert("face_vomiting", "\u{1F92E}");
    m.insert("face_with_hand_over_mouth", "\u{1F92D}");
    m.insert("face_with_head-bandage", "\u{1F915}");
    m.insert("face_with_medical_mask", "\u{1F637}");
    m.insert("face_with_monocle", "\u{1F9D0}");
    m.insert("face_with_open_mouth", "\u{1F62E}");
    m.insert("face_with_raised_eyebrow", "\u{1F928}");
    m.insert("face_with_rolling_eyes", "\u{1F644}");
    m.insert("face_with_steam_from_nose", "\u{1F624}");
    m.insert("face_with_symbols_on_mouth", "\u{1F92C}");
    m.insert("face_with_tears_of_joy", "\u{1F602}");
    m.insert("face_with_thermometer", "\u{1F912}");
    m.insert("face_with_tongue", "\u{1F61B}");
    m.insert("face_without_mouth", "\u{1F636}");
    m.insert("factory", "\u{1F3ED}");
    m.insert("fairy", "\u{1F9DA}");
    m.insert("fairy_dark_skin_tone", "\u{1F9DA}\u{1F3FF}");
    m.insert("fairy_light_skin_tone", "\u{1F9DA}\u{1F3FB}");
    m.insert("fairy_medium-dark_skin_tone", "\u{1F9DA}\u{1F3FE}");
    m.insert("fairy_medium-light_skin_tone", "\u{1F9DA}\u{1F3FC}");
    m.insert("fairy_medium_skin_tone", "\u{1F9DA}\u{1F3FD}");
    m.insert("falafel", "\u{1F9C6}");
    m.insert("fallen_leaf", "\u{1F342}");
    m.insert("family", "\u{1F46A}");
    m.insert("family_man_boy", "\u{1F468}\u{200D}\u{1F466}");
    m.insert(
        "family_man_boy_boy",
        "\u{1F468}\u{200D}\u{1F466}\u{200D}\u{1F466}",
    );
    m.insert("family_man_girl", "\u{1F468}\u{200D}\u{1F467}");
    m.insert(
        "family_man_girl_boy",
        "\u{1F468}\u{200D}\u{1F467}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_girl_girl",
        "\u{1F468}\u{200D}\u{1F467}\u{200D}\u{1F467}",
    );
    m.insert(
        "family_man_man_boy",
        "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_man_boy_boy",
        "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F466}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_man_girl",
        "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F467}",
    );
    m.insert(
        "family_man_man_girl_boy",
        "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F467}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_man_girl_girl",
        "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F467}\u{200D}\u{1F467}",
    );
    m.insert(
        "family_man_woman_boy",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_woman_boy_boy",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_woman_girl",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
    );
    m.insert(
        "family_man_woman_girl_boy",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_man_woman_girl_girl",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F467}",
    );
    m.insert("family_woman_boy", "\u{1F469}\u{200D}\u{1F466}");
    m.insert(
        "family_woman_boy_boy",
        "\u{1F469}\u{200D}\u{1F466}\u{200D}\u{1F466}",
    );
    m.insert("family_woman_girl", "\u{1F469}\u{200D}\u{1F467}");
    m.insert(
        "family_woman_girl_boy",
        "\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_woman_girl_girl",
        "\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F467}",
    );
    m.insert(
        "family_woman_woman_boy",
        "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_woman_woman_boy_boy",
        "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F466}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_woman_woman_girl",
        "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}",
    );
    m.insert(
        "family_woman_woman_girl_boy",
        "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
    );
    m.insert(
        "family_woman_woman_girl_girl",
        "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F467}",
    );
    m.insert("fast-forward_button", "\u{23E9}");
    m.insert("fast_down_button", "\u{23EC}");
    m.insert("fast_reverse_button", "\u{23EA}");
    m.insert("fast_up_button", "\u{23EB}");
    m.insert("fax_machine", "\u{1F4E0}");
    m.insert("fearful_face", "\u{1F628}");
    m.insert("female_sign", "\u{2640}");
    m.insert("ferris_wheel", "\u{1F3A1}");
    m.insert("ferry", "\u{26F4}");
    m.insert("field_hockey", "\u{1F3D1}");
    m.insert("file_cabinet", "\u{1F5C4}");
    m.insert("file_folder", "\u{1F4C1}");
    m.insert("film_frames", "\u{1F39E}");
    m.insert("film_projector", "\u{1F4FD}");
    m.insert("fire", "\u{1F525}");
    m.insert("fire_extinguisher", "\u{1F9EF}");
    m.insert("firecracker", "\u{1F9E8}");
    m.insert("fire_engine", "\u{1F692}");
    m.insert("fireworks", "\u{1F386}");
    m.insert("first_quarter_moon", "\u{1F313}");
    m.insert("first_quarter_moon_face", "\u{1F31B}");
    m.insert("fish", "\u{1F41F}");
    m.insert("fish_cake_with_swirl", "\u{1F365}");
    m.insert("fishing_pole", "\u{1F3A3}");
    m.insert("five-thirty", "\u{1F560}");
    m.insert("five_o’clock", "\u{1F554}");
    m.insert("flag_in_hole", "\u{26F3}");
    m.insert("flamingo", "\u{1F9A9}");
    m.insert("flashlight", "\u{1F526}");
    m.insert("flat_shoe", "\u{1F97F}");
    m.insert("fleur-de-lis", "\u{269C}");
    m.insert("flexed_biceps", "\u{1F4AA}");
    m.insert("flexed_biceps_dark_skin_tone", "\u{1F4AA}\u{1F3FF}");
    m.insert("flexed_biceps_light_skin_tone", "\u{1F4AA}\u{1F3FB}");
    m.insert("flexed_biceps_medium-dark_skin_tone", "\u{1F4AA}\u{1F3FE}");
    m.insert("flexed_biceps_medium-light_skin_tone", "\u{1F4AA}\u{1F3FC}");
    m.insert("flexed_biceps_medium_skin_tone", "\u{1F4AA}\u{1F3FD}");
    m.insert("floppy_disk", "\u{1F4BE}");
    m.insert("flower_playing_cards", "\u{1F3B4}");
    m.insert("flushed_face", "\u{1F633}");
    m.insert("flying_disc", "\u{1F94F}");
    m.insert("flying_saucer", "\u{1F6F8}");
    m.insert("fog", "\u{1F32B}");
    m.insert("foggy", "\u{1F301}");
    m.insert("folded_hands", "\u{1F64F}");
    m.insert("folded_hands_dark_skin_tone", "\u{1F64F}\u{1F3FF}");
    m.insert("folded_hands_light_skin_tone", "\u{1F64F}\u{1F3FB}");
    m.insert("folded_hands_medium-dark_skin_tone", "\u{1F64F}\u{1F3FE}");
    m.insert("folded_hands_medium-light_skin_tone", "\u{1F64F}\u{1F3FC}");
    m.insert("folded_hands_medium_skin_tone", "\u{1F64F}\u{1F3FD}");
    m.insert("foot", "\u{1F9B6}");
    m.insert("footprints", "\u{1F463}");
    m.insert("fork_and_knife", "\u{1F374}");
    m.insert("fork_and_knife_with_plate", "\u{1F37D}");
    m.insert("fortune_cookie", "\u{1F960}");
    m.insert("fountain", "\u{26F2}");
    m.insert("fountain_pen", "\u{1F58B}");
    m.insert("four-thirty", "\u{1F55F}");
    m.insert("four_leaf_clover", "\u{1F340}");
    m.insert("four_o’clock", "\u{1F553}");
    m.insert("fox_face", "\u{1F98A}");
    m.insert("framed_picture", "\u{1F5BC}");
    m.insert("french_fries", "\u{1F35F}");
    m.insert("fried_shrimp", "\u{1F364}");
    m.insert("frog_face", "\u{1F438}");
    m.insert("front-facing_baby_chick", "\u{1F425}");
    m.insert("frowning_face", "\u{2639}");
    m.insert("frowning_face_with_open_mouth", "\u{1F626}");
    m.insert("fuel_pump", "\u{26FD}");
    m.insert("full_moon", "\u{1F315}");
    m.insert("full_moon_face", "\u{1F31D}");
    m.insert("funeral_urn", "\u{26B1}");
    m.insert("game_die", "\u{1F3B2}");
    m.insert("garlic", "\u{1F9C4}");
    m.insert("gear", "\u{2699}");
    m.insert("gem_stone", "\u{1F48E}");
    m.insert("genie", "\u{1F9DE}");
    m.insert("ghost", "\u{1F47B}");
    m.insert("giraffe", "\u{1F992}");
    m.insert("girl", "\u{1F467}");
    m.insert("girl_dark_skin_tone", "\u{1F467}\u{1F3FF}");
    m.insert("girl_light_skin_tone", "\u{1F467}\u{1F3FB}");
    m.insert("girl_medium-dark_skin_tone", "\u{1F467}\u{1F3FE}");
    m.insert("girl_medium-light_skin_tone", "\u{1F467}\u{1F3FC}");
    m.insert("girl_medium_skin_tone", "\u{1F467}\u{1F3FD}");
    m.insert("glass_of_milk", "\u{1F95B}");
    m.insert("glasses", "\u{1F453}");
    m.insert("globe_showing_americas", "\u{1F30E}");
    m.insert("globe_showing_asia-australia", "\u{1F30F}");
    m.insert("globe_showing_europe-africa", "\u{1F30D}");
    m.insert("globe_with_meridians", "\u{1F310}");
    m.insert("gloves", "\u{1F9E4}");
    m.insert("glowing_star", "\u{1F31F}");
    m.insert("goal_net", "\u{1F945}");
    m.insert("goat", "\u{1F410}");
    m.insert("goblin", "\u{1F47A}");
    m.insert("goggles", "\u{1F97D}");
    m.insert("gorilla", "\u{1F98D}");
    m.insert("graduation_cap", "\u{1F393}");
    m.insert("grapes", "\u{1F347}");
    m.insert("green_apple", "\u{1F34F}");
    m.insert("green_book", "\u{1F4D7}");
    m.insert("green_circle", "\u{1F7E2}");
    m.insert("green_heart", "\u{1F49A}");
    m.insert("green_salad", "\u{1F957}");
    m.insert("green_square", "\u{1F7E9}");
    m.insert("grimacing_face", "\u{1F62C}");
    m.insert("grinning_cat_face", "\u{1F63A}");
    m.insert("grinning_cat_face_with_smiling_eyes", "\u{1F638}");
    m.insert("grinning_face", "\u{1F600}");
    m.insert("grinning_face_with_big_eyes", "\u{1F603}");
    m.insert("grinning_face_with_smiling_eyes", "\u{1F604}");
    m.insert("grinning_face_with_sweat", "\u{1F605}");
    m.insert("grinning_squinting_face", "\u{1F606}");
    m.insert("growing_heart", "\u{1F497}");
    m.insert("guard", "\u{1F482}");
    m.insert("guard_dark_skin_tone", "\u{1F482}\u{1F3FF}");
    m.insert("guard_light_skin_tone", "\u{1F482}\u{1F3FB}");
    m.insert("guard_medium-dark_skin_tone", "\u{1F482}\u{1F3FE}");
    m.insert("guard_medium-light_skin_tone", "\u{1F482}\u{1F3FC}");
    m.insert("guard_medium_skin_tone", "\u{1F482}\u{1F3FD}");
    m.insert("guide_dog", "\u{1F9AE}");
    m.insert("guitar", "\u{1F3B8}");
    m.insert("hamburger", "\u{1F354}");
    m.insert("hammer", "\u{1F528}");
    m.insert("hammer_and_pick", "\u{2692}");
    m.insert("hammer_and_wrench", "\u{1F6E0}");
    m.insert("hamster_face", "\u{1F439}");
    m.insert("hand_with_fingers_splayed", "\u{1F590}");
    m.insert(
        "hand_with_fingers_splayed_dark_skin_tone",
        "\u{1F590}\u{1F3FF}",
    );
    m.insert(
        "hand_with_fingers_splayed_light_skin_tone",
        "\u{1F590}\u{1F3FB}",
    );
    m.insert(
        "hand_with_fingers_splayed_medium-dark_skin_tone",
        "\u{1F590}\u{1F3FE}",
    );
    m.insert(
        "hand_with_fingers_splayed_medium-light_skin_tone",
        "\u{1F590}\u{1F3FC}",
    );
    m.insert(
        "hand_with_fingers_splayed_medium_skin_tone",
        "\u{1F590}\u{1F3FD}",
    );
    m.insert("handbag", "\u{1F45C}");
    m.insert("handshake", "\u{1F91D}");
    m.insert("hatching_chick", "\u{1F423}");
    m.insert("headphone", "\u{1F3A7}");
    m.insert("hear-no-evil_monkey", "\u{1F649}");
    m.insert("heart_decoration", "\u{1F49F}");
    m.insert("heart_suit", "\u{2665}");
    m.insert("heart_with_arrow", "\u{1F498}");
    m.insert("heart_with_ribbon", "\u{1F49D}");
    m.insert("heavy_check_mark", "\u{2714}");
    m.insert("heavy_division_sign", "\u{2797}");
    m.insert("heavy_dollar_sign", "\u{1F4B2}");
    m.insert("heavy_heart_exclamation", "\u{2763}");
    m.insert("heavy_large_circle", "\u{2B55}");
    m.insert("heavy_minus_sign", "\u{2796}");
    m.insert("heavy_multiplication_x", "\u{2716}");
    m.insert("heavy_plus_sign", "\u{2795}");
    m.insert("hedgehog", "\u{1F994}");
    m.insert("helicopter", "\u{1F681}");
    m.insert("herb", "\u{1F33F}");
    m.insert("hibiscus", "\u{1F33A}");
    m.insert("high-heeled_shoe", "\u{1F460}");
    m.insert("high-speed_train", "\u{1F684}");
    m.insert("high_voltage", "\u{26A1}");
    m.insert("hiking_boot", "\u{1F97E}");
    m.insert("hindu_temple", "\u{1F6D5}");
    m.insert("hippopotamus", "\u{1F99B}");
    m.insert("hole", "\u{1F573}");
    m.insert("honey_pot", "\u{1F36F}");
    m.insert("honeybee", "\u{1F41D}");
    m.insert("horizontal_traffic_light", "\u{1F6A5}");
    m.insert("horse", "\u{1F434}");
    m.insert("horse_face", "\u{1F434}");
    m.insert("horse_racing", "\u{1F3C7}");
    m.insert("horse_racing_dark_skin_tone", "\u{1F3C7}\u{1F3FF}");
    m.insert("horse_racing_light_skin_tone", "\u{1F3C7}\u{1F3FB}");
    m.insert("horse_racing_medium-dark_skin_tone", "\u{1F3C7}\u{1F3FE}");
    m.insert("horse_racing_medium-light_skin_tone", "\u{1F3C7}\u{1F3FC}");
    m.insert("horse_racing_medium_skin_tone", "\u{1F3C7}\u{1F3FD}");
    m.insert("hospital", "\u{1F3E5}");
    m.insert("hot_beverage", "\u{2615}");
    m.insert("hot_dog", "\u{1F32D}");
    m.insert("hot_face", "\u{1F975}");
    m.insert("hot_pepper", "\u{1F336}");
    m.insert("hot_springs", "\u{2668}");
    m.insert("hotel", "\u{1F3E8}");
    m.insert("hourglass_done", "\u{231B}");
    m.insert("hourglass_not_done", "\u{23F3}");
    m.insert("house", "\u{1F3E0}");
    m.insert("house_with_garden", "\u{1F3E1}");
    m.insert("houses", "\u{1F3D8}");
    m.insert("hugging_face", "\u{1F917}");
    m.insert("hundred_points", "\u{1F4AF}");
    m.insert("hushed_face", "\u{1F62F}");
    m.insert("ice", "\u{1F9CA}");
    m.insert("ice_cream", "\u{1F368}");
    m.insert("ice_hockey", "\u{1F3D2}");
    m.insert("ice_skate", "\u{26F8}");
    m.insert("inbox_tray", "\u{1F4E5}");
    m.insert("incoming_envelope", "\u{1F4E8}");
    m.insert("index_pointing_up", "\u{261D}");
    m.insert("index_pointing_up_dark_skin_tone", "\u{261D}\u{1F3FF}");
    m.insert("index_pointing_up_light_skin_tone", "\u{261D}\u{1F3FB}");
    m.insert(
        "index_pointing_up_medium-dark_skin_tone",
        "\u{261D}\u{1F3FE}",
    );
    m.insert(
        "index_pointing_up_medium-light_skin_tone",
        "\u{261D}\u{1F3FC}",
    );
    m.insert("index_pointing_up_medium_skin_tone", "\u{261D}\u{1F3FD}");
    m.insert("infinity", "\u{267E}");
    m.insert("information", "\u{2139}");
    m.insert("input_latin_letters", "\u{1F524}");
    m.insert("input_latin_lowercase", "\u{1F521}");
    m.insert("input_latin_uppercase", "\u{1F520}");
    m.insert("input_numbers", "\u{1F522}");
    m.insert("input_symbols", "\u{1F523}");
    m.insert("jack-o-lantern", "\u{1F383}");
    m.insert("jeans", "\u{1F456}");
    m.insert("jigsaw", "\u{1F9E9}");
    m.insert("joker", "\u{1F0CF}");
    m.insert("joystick", "\u{1F579}");
    m.insert("kaaba", "\u{1F54B}");
    m.insert("kangaroo", "\u{1F998}");
    m.insert("key", "\u{1F511}");
    m.insert("keyboard", "\u{2328}");
    m.insert("keycap_#", "#\u{FE0F}\u{20E3}");
    m.insert("keycap_*", "*\u{FE0F}\u{20E3}");
    m.insert("keycap_0", "0\u{FE0F}\u{20E3}");
    m.insert("keycap_1", "1\u{FE0F}\u{20E3}");
    m.insert("keycap_10", "\u{1F51F}");
    m.insert("keycap_2", "2\u{FE0F}\u{20E3}");
    m.insert("keycap_3", "3\u{FE0F}\u{20E3}");
    m.insert("keycap_4", "4\u{FE0F}\u{20E3}");
    m.insert("keycap_5", "5\u{FE0F}\u{20E3}");
    m.insert("keycap_6", "6\u{FE0F}\u{20E3}");
    m.insert("keycap_7", "7\u{FE0F}\u{20E3}");
    m.insert("keycap_8", "8\u{FE0F}\u{20E3}");
    m.insert("keycap_9", "9\u{FE0F}\u{20E3}");
    m.insert("kick_scooter", "\u{1F6F4}");
    m.insert("kimono", "\u{1F458}");
    m.insert("kiss", "\u{1F48B}");
    m.insert(
        "kiss_man_man",
        "\u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F48B}\u{200D}\u{1F468}",
    );
    m.insert("kiss_mark", "\u{1F48B}");
    m.insert(
        "kiss_woman_man",
        "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F48B}\u{200D}\u{1F468}",
    );
    m.insert(
        "kiss_woman_woman",
        "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F48B}\u{200D}\u{1F469}",
    );
    m.insert("kissing_cat_face", "\u{1F63D}");
    m.insert("kissing_face", "\u{1F617}");
    m.insert("kissing_face_with_closed_eyes", "\u{1F61A}");
    m.insert("kissing_face_with_smiling_eyes", "\u{1F619}");
    m.insert("kitchen_knife", "\u{1F52A}");
    m.insert("kite", "\u{1FA81}");
    m.insert("kiwi_fruit", "\u{1F95D}");
    m.insert("koala", "\u{1F428}");
    m.insert("lab_coat", "\u{1F97C}");
    m.insert("label", "\u{1F3F7}");
    m.insert("lacrosse", "\u{1F94D}");
    m.insert("lady_beetle", "\u{1F41E}");
    m.insert("laptop_computer", "\u{1F4BB}");
    m.insert("large_blue_diamond", "\u{1F537}");
    m.insert("large_orange_diamond", "\u{1F536}");
    m.insert("last_quarter_moon", "\u{1F317}");
    m.insert("last_quarter_moon_face", "\u{1F31C}");
    m.insert("last_track_button", "\u{23EE}");
    m.insert("latin_cross", "\u{271D}");
    m.insert("leaf_fluttering_in_wind", "\u{1F343}");
    m.insert("leafy_green", "\u{1F96C}");
    m.insert("ledger", "\u{1F4D2}");
    m.insert("left-facing_fist", "\u{1F91B}");
    m.insert("left-facing_fist_dark_skin_tone", "\u{1F91B}\u{1F3FF}");
    m.insert("left-facing_fist_light_skin_tone", "\u{1F91B}\u{1F3FB}");
    m.insert(
        "left-facing_fist_medium-dark_skin_tone",
        "\u{1F91B}\u{1F3FE}",
    );
    m.insert(
        "left-facing_fist_medium-light_skin_tone",
        "\u{1F91B}\u{1F3FC}",
    );
    m.insert("left-facing_fist_medium_skin_tone", "\u{1F91B}\u{1F3FD}");
    m.insert("left-right_arrow", "\u{2194}");
    m.insert("left_arrow", "\u{2B05}");
    m.insert("left_arrow_curving_right", "\u{21AA}");
    m.insert("left_luggage", "\u{1F6C5}");
    m.insert("left_speech_bubble", "\u{1F5E8}");
    m.insert("leg", "\u{1F9B5}");
    m.insert("lemon", "\u{1F34B}");
    m.insert("leopard", "\u{1F406}");
    m.insert("level_slider", "\u{1F39A}");
    m.insert("light_bulb", "\u{1F4A1}");
    m.insert("light_rail", "\u{1F688}");
    m.insert("link", "\u{1F517}");
    m.insert("linked_paperclips", "\u{1F587}");
    m.insert("lion_face", "\u{1F981}");
    m.insert("lipstick", "\u{1F484}");
    m.insert("litter_in_bin_sign", "\u{1F6AE}");
    m.insert("lizard", "\u{1F98E}");
    m.insert("llama", "\u{1F999}");
    m.insert("lobster", "\u{1F99E}");
    m.insert("locked", "\u{1F512}");
    m.insert("locked_with_key", "\u{1F510}");
    m.insert("locked_with_pen", "\u{1F50F}");
    m.insert("locomotive", "\u{1F682}");
    m.insert("lollipop", "\u{1F36D}");
    m.insert("lotion_bottle", "\u{1F9F4}");
    m.insert("loudly_crying_face", "\u{1F62D}");
    m.insert("loudspeaker", "\u{1F4E2}");
    m.insert("love-you_gesture", "\u{1F91F}");
    m.insert("love-you_gesture_dark_skin_tone", "\u{1F91F}\u{1F3FF}");
    m.insert("love-you_gesture_light_skin_tone", "\u{1F91F}\u{1F3FB}");
    m.insert(
        "love-you_gesture_medium-dark_skin_tone",
        "\u{1F91F}\u{1F3FE}",
    );
    m.insert(
        "love-you_gesture_medium-light_skin_tone",
        "\u{1F91F}\u{1F3FC}",
    );
    m.insert("love-you_gesture_medium_skin_tone", "\u{1F91F}\u{1F3FD}");
    m.insert("love_hotel", "\u{1F3E9}");
    m.insert("love_letter", "\u{1F48C}");
    m.insert("luggage", "\u{1F9F3}");
    m.insert("lying_face", "\u{1F925}");
    m.insert("mage", "\u{1F9D9}");
    m.insert("mage_dark_skin_tone", "\u{1F9D9}\u{1F3FF}");
    m.insert("mage_light_skin_tone", "\u{1F9D9}\u{1F3FB}");
    m.insert("mage_medium-dark_skin_tone", "\u{1F9D9}\u{1F3FE}");
    m.insert("mage_medium-light_skin_tone", "\u{1F9D9}\u{1F3FC}");
    m.insert("mage_medium_skin_tone", "\u{1F9D9}\u{1F3FD}");
    m.insert("magnet", "\u{1F9F2}");
    m.insert("magnifying_glass_tilted_left", "\u{1F50D}");
    m.insert("magnifying_glass_tilted_right", "\u{1F50E}");
    m.insert("mahjong_red_dragon", "\u{1F004}");
    m.insert("male_sign", "\u{2642}");
    m.insert("man", "\u{1F468}");
    m.insert("man_and_woman_holding_hands", "\u{1F46B}");
    m.insert("man_artist", "\u{1F468}\u{200D}\u{1F3A8}");
    m.insert(
        "man_artist_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "man_artist_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "man_artist_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "man_artist_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "man_artist_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F3A8}",
    );
    m.insert("man_astronaut", "\u{1F468}\u{200D}\u{1F680}");
    m.insert(
        "man_astronaut_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F680}",
    );
    m.insert(
        "man_astronaut_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F680}",
    );
    m.insert(
        "man_astronaut_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F680}",
    );
    m.insert(
        "man_astronaut_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F680}",
    );
    m.insert(
        "man_astronaut_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F680}",
    );
    m.insert("man_biking", "\u{1F6B4}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_biking_dark_skin_tone",
        "\u{1F6B4}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_biking_light_skin_tone",
        "\u{1F6B4}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_biking_medium-dark_skin_tone",
        "\u{1F6B4}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_biking_medium-light_skin_tone",
        "\u{1F6B4}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_biking_medium_skin_tone",
        "\u{1F6B4}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bouncing_ball",
        "\u{26F9}\u{FE0F}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bouncing_ball_dark_skin_tone",
        "\u{26F9}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bouncing_ball_light_skin_tone",
        "\u{26F9}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bouncing_ball_medium-dark_skin_tone",
        "\u{26F9}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bouncing_ball_medium-light_skin_tone",
        "\u{26F9}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bouncing_ball_medium_skin_tone",
        "\u{26F9}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_bowing", "\u{1F647}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_bowing_dark_skin_tone",
        "\u{1F647}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bowing_light_skin_tone",
        "\u{1F647}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bowing_medium-dark_skin_tone",
        "\u{1F647}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bowing_medium-light_skin_tone",
        "\u{1F647}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_bowing_medium_skin_tone",
        "\u{1F647}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_cartwheeling", "\u{1F938}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_cartwheeling_dark_skin_tone",
        "\u{1F938}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_cartwheeling_light_skin_tone",
        "\u{1F938}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_cartwheeling_medium-dark_skin_tone",
        "\u{1F938}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_cartwheeling_medium-light_skin_tone",
        "\u{1F938}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_cartwheeling_medium_skin_tone",
        "\u{1F938}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_climbing", "\u{1F9D7}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_climbing_dark_skin_tone",
        "\u{1F9D7}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_climbing_light_skin_tone",
        "\u{1F9D7}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_climbing_medium-dark_skin_tone",
        "\u{1F9D7}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_climbing_medium-light_skin_tone",
        "\u{1F9D7}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_climbing_medium_skin_tone",
        "\u{1F9D7}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_construction_worker",
        "\u{1F477}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_construction_worker_dark_skin_tone",
        "\u{1F477}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_construction_worker_light_skin_tone",
        "\u{1F477}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_construction_worker_medium-dark_skin_tone",
        "\u{1F477}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_construction_worker_medium-light_skin_tone",
        "\u{1F477}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_construction_worker_medium_skin_tone",
        "\u{1F477}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_cook", "\u{1F468}\u{200D}\u{1F373}");
    m.insert(
        "man_cook_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F373}",
    );
    m.insert(
        "man_cook_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F373}",
    );
    m.insert(
        "man_cook_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F373}",
    );
    m.insert(
        "man_cook_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F373}",
    );
    m.insert(
        "man_cook_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F373}",
    );
    m.insert("man_dancing", "\u{1F57A}");
    m.insert("man_dancing_dark_skin_tone", "\u{1F57A}\u{1F3FF}");
    m.insert("man_dancing_light_skin_tone", "\u{1F57A}\u{1F3FB}");
    m.insert("man_dancing_medium-dark_skin_tone", "\u{1F57A}\u{1F3FE}");
    m.insert("man_dancing_medium-light_skin_tone", "\u{1F57A}\u{1F3FC}");
    m.insert("man_dancing_medium_skin_tone", "\u{1F57A}\u{1F3FD}");
    m.insert("man_dark_skin_tone", "\u{1F468}\u{1F3FF}");
    m.insert("man_detective", "\u{1F575}\u{FE0F}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_detective_dark_skin_tone",
        "\u{1F575}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_detective_light_skin_tone",
        "\u{1F575}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_detective_medium-dark_skin_tone",
        "\u{1F575}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_detective_medium-light_skin_tone",
        "\u{1F575}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_detective_medium_skin_tone",
        "\u{1F575}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_elf", "\u{1F9DD}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_elf_dark_skin_tone",
        "\u{1F9DD}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_elf_light_skin_tone",
        "\u{1F9DD}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_elf_medium-dark_skin_tone",
        "\u{1F9DD}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_elf_medium-light_skin_tone",
        "\u{1F9DD}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_elf_medium_skin_tone",
        "\u{1F9DD}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_facepalming", "\u{1F926}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_facepalming_dark_skin_tone",
        "\u{1F926}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_facepalming_light_skin_tone",
        "\u{1F926}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_facepalming_medium-dark_skin_tone",
        "\u{1F926}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_facepalming_medium-light_skin_tone",
        "\u{1F926}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_facepalming_medium_skin_tone",
        "\u{1F926}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_factory_worker", "\u{1F468}\u{200D}\u{1F3ED}");
    m.insert(
        "man_factory_worker_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "man_factory_worker_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "man_factory_worker_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "man_factory_worker_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "man_factory_worker_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F3ED}",
    );
    m.insert("man_fairy", "\u{1F9DA}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_fairy_dark_skin_tone",
        "\u{1F9DA}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_fairy_light_skin_tone",
        "\u{1F9DA}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_fairy_medium-dark_skin_tone",
        "\u{1F9DA}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_fairy_medium-light_skin_tone",
        "\u{1F9DA}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_fairy_medium_skin_tone",
        "\u{1F9DA}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_farmer", "\u{1F468}\u{200D}\u{1F33E}");
    m.insert(
        "man_farmer_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F33E}",
    );
    m.insert(
        "man_farmer_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F33E}",
    );
    m.insert(
        "man_farmer_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F33E}",
    );
    m.insert(
        "man_farmer_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F33E}",
    );
    m.insert(
        "man_farmer_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F33E}",
    );
    m.insert("man_firefighter", "\u{1F468}\u{200D}\u{1F692}");
    m.insert(
        "man_firefighter_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F692}",
    );
    m.insert(
        "man_firefighter_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F692}",
    );
    m.insert(
        "man_firefighter_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F692}",
    );
    m.insert(
        "man_firefighter_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F692}",
    );
    m.insert(
        "man_firefighter_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F692}",
    );
    m.insert("man_frowning", "\u{1F64D}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_frowning_dark_skin_tone",
        "\u{1F64D}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_frowning_light_skin_tone",
        "\u{1F64D}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_frowning_medium-dark_skin_tone",
        "\u{1F64D}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_frowning_medium-light_skin_tone",
        "\u{1F64D}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_frowning_medium_skin_tone",
        "\u{1F64D}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_genie", "\u{1F9DE}\u{200D}\u{2642}\u{FE0F}");
    m.insert("man_gesturing_no", "\u{1F645}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_gesturing_no_dark_skin_tone",
        "\u{1F645}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_no_light_skin_tone",
        "\u{1F645}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_no_medium-dark_skin_tone",
        "\u{1F645}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_no_medium-light_skin_tone",
        "\u{1F645}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_no_medium_skin_tone",
        "\u{1F645}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_gesturing_ok", "\u{1F646}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_gesturing_ok_dark_skin_tone",
        "\u{1F646}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_ok_light_skin_tone",
        "\u{1F646}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_ok_medium-dark_skin_tone",
        "\u{1F646}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_ok_medium-light_skin_tone",
        "\u{1F646}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_gesturing_ok_medium_skin_tone",
        "\u{1F646}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_getting_haircut", "\u{1F487}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_getting_haircut_dark_skin_tone",
        "\u{1F487}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_haircut_light_skin_tone",
        "\u{1F487}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_haircut_medium-dark_skin_tone",
        "\u{1F487}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_haircut_medium-light_skin_tone",
        "\u{1F487}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_haircut_medium_skin_tone",
        "\u{1F487}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_getting_massage", "\u{1F486}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_getting_massage_dark_skin_tone",
        "\u{1F486}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_massage_light_skin_tone",
        "\u{1F486}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_massage_medium-dark_skin_tone",
        "\u{1F486}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_massage_medium-light_skin_tone",
        "\u{1F486}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_getting_massage_medium_skin_tone",
        "\u{1F486}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_golfing", "\u{1F3CC}\u{FE0F}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_golfing_dark_skin_tone",
        "\u{1F3CC}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_golfing_light_skin_tone",
        "\u{1F3CC}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_golfing_medium-dark_skin_tone",
        "\u{1F3CC}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_golfing_medium-light_skin_tone",
        "\u{1F3CC}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_golfing_medium_skin_tone",
        "\u{1F3CC}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_guard", "\u{1F482}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_guard_dark_skin_tone",
        "\u{1F482}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_guard_light_skin_tone",
        "\u{1F482}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_guard_medium-dark_skin_tone",
        "\u{1F482}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_guard_medium-light_skin_tone",
        "\u{1F482}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_guard_medium_skin_tone",
        "\u{1F482}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_health_worker", "\u{1F468}\u{200D}\u{2695}\u{FE0F}");
    m.insert(
        "man_health_worker_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "man_health_worker_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "man_health_worker_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "man_health_worker_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "man_health_worker_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert("man_in_lotus_position", "\u{1F9D8}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_in_lotus_position_dark_skin_tone",
        "\u{1F9D8}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_lotus_position_light_skin_tone",
        "\u{1F9D8}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_lotus_position_medium-dark_skin_tone",
        "\u{1F9D8}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_lotus_position_medium-light_skin_tone",
        "\u{1F9D8}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_lotus_position_medium_skin_tone",
        "\u{1F9D8}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_in_manual_wheelchair", "\u{1F468}\u{200D}\u{1F9BD}");
    m.insert("man_in_motorized_wheelchair", "\u{1F468}\u{200D}\u{1F9BC}");
    m.insert("man_in_steamy_room", "\u{1F9D6}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_in_steamy_room_dark_skin_tone",
        "\u{1F9D6}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_steamy_room_light_skin_tone",
        "\u{1F9D6}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_steamy_room_medium-dark_skin_tone",
        "\u{1F9D6}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_steamy_room_medium-light_skin_tone",
        "\u{1F9D6}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_in_steamy_room_medium_skin_tone",
        "\u{1F9D6}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_in_suit_levitating", "\u{1F574}");
    m.insert(
        "man_in_suit_levitating_dark_skin_tone",
        "\u{1F574}\u{1F3FF}",
    );
    m.insert(
        "man_in_suit_levitating_light_skin_tone",
        "\u{1F574}\u{1F3FB}",
    );
    m.insert(
        "man_in_suit_levitating_medium-dark_skin_tone",
        "\u{1F574}\u{1F3FE}",
    );
    m.insert(
        "man_in_suit_levitating_medium-light_skin_tone",
        "\u{1F574}\u{1F3FC}",
    );
    m.insert(
        "man_in_suit_levitating_medium_skin_tone",
        "\u{1F574}\u{1F3FD}",
    );
    m.insert("man_in_tuxedo", "\u{1F935}");
    m.insert("man_in_tuxedo_dark_skin_tone", "\u{1F935}\u{1F3FF}");
    m.insert("man_in_tuxedo_light_skin_tone", "\u{1F935}\u{1F3FB}");
    m.insert("man_in_tuxedo_medium-dark_skin_tone", "\u{1F935}\u{1F3FE}");
    m.insert("man_in_tuxedo_medium-light_skin_tone", "\u{1F935}\u{1F3FC}");
    m.insert("man_in_tuxedo_medium_skin_tone", "\u{1F935}\u{1F3FD}");
    m.insert("man_judge", "\u{1F468}\u{200D}\u{2696}\u{FE0F}");
    m.insert(
        "man_judge_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "man_judge_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "man_judge_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "man_judge_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "man_judge_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert("man_juggling", "\u{1F939}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_juggling_dark_skin_tone",
        "\u{1F939}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_juggling_light_skin_tone",
        "\u{1F939}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_juggling_medium-dark_skin_tone",
        "\u{1F939}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_juggling_medium-light_skin_tone",
        "\u{1F939}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_juggling_medium_skin_tone",
        "\u{1F939}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_lifting_weights",
        "\u{1F3CB}\u{FE0F}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_lifting_weights_dark_skin_tone",
        "\u{1F3CB}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_lifting_weights_light_skin_tone",
        "\u{1F3CB}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_lifting_weights_medium-dark_skin_tone",
        "\u{1F3CB}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_lifting_weights_medium-light_skin_tone",
        "\u{1F3CB}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_lifting_weights_medium_skin_tone",
        "\u{1F3CB}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_light_skin_tone", "\u{1F468}\u{1F3FB}");
    m.insert("man_mage", "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_mage_dark_skin_tone",
        "\u{1F9D9}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mage_light_skin_tone",
        "\u{1F9D9}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mage_medium-dark_skin_tone",
        "\u{1F9D9}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mage_medium-light_skin_tone",
        "\u{1F9D9}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mage_medium_skin_tone",
        "\u{1F9D9}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_mechanic", "\u{1F468}\u{200D}\u{1F527}");
    m.insert(
        "man_mechanic_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F527}",
    );
    m.insert(
        "man_mechanic_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F527}",
    );
    m.insert(
        "man_mechanic_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F527}",
    );
    m.insert(
        "man_mechanic_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F527}",
    );
    m.insert(
        "man_mechanic_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F527}",
    );
    m.insert("man_medium-dark_skin_tone", "\u{1F468}\u{1F3FE}");
    m.insert("man_medium-light_skin_tone", "\u{1F468}\u{1F3FC}");
    m.insert("man_medium_skin_tone", "\u{1F468}\u{1F3FD}");
    m.insert("man_mountain_biking", "\u{1F6B5}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_mountain_biking_dark_skin_tone",
        "\u{1F6B5}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mountain_biking_light_skin_tone",
        "\u{1F6B5}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mountain_biking_medium-dark_skin_tone",
        "\u{1F6B5}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mountain_biking_medium-light_skin_tone",
        "\u{1F6B5}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_mountain_biking_medium_skin_tone",
        "\u{1F6B5}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_office_worker", "\u{1F468}\u{200D}\u{1F4BC}");
    m.insert(
        "man_office_worker_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "man_office_worker_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "man_office_worker_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "man_office_worker_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "man_office_worker_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F4BC}",
    );
    m.insert("man_pilot", "\u{1F468}\u{200D}\u{2708}\u{FE0F}");
    m.insert(
        "man_pilot_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "man_pilot_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "man_pilot_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "man_pilot_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "man_pilot_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert("man_playing_handball", "\u{1F93E}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_playing_handball_dark_skin_tone",
        "\u{1F93E}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_handball_light_skin_tone",
        "\u{1F93E}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_handball_medium-dark_skin_tone",
        "\u{1F93E}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_handball_medium-light_skin_tone",
        "\u{1F93E}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_handball_medium_skin_tone",
        "\u{1F93E}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_water_polo",
        "\u{1F93D}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_water_polo_dark_skin_tone",
        "\u{1F93D}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_water_polo_light_skin_tone",
        "\u{1F93D}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_water_polo_medium-dark_skin_tone",
        "\u{1F93D}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_water_polo_medium-light_skin_tone",
        "\u{1F93D}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_playing_water_polo_medium_skin_tone",
        "\u{1F93D}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_police_officer", "\u{1F46E}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_police_officer_dark_skin_tone",
        "\u{1F46E}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_police_officer_light_skin_tone",
        "\u{1F46E}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_police_officer_medium-dark_skin_tone",
        "\u{1F46E}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_police_officer_medium-light_skin_tone",
        "\u{1F46E}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_police_officer_medium_skin_tone",
        "\u{1F46E}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_pouting", "\u{1F64E}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_pouting_dark_skin_tone",
        "\u{1F64E}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_pouting_light_skin_tone",
        "\u{1F64E}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_pouting_medium-dark_skin_tone",
        "\u{1F64E}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_pouting_medium-light_skin_tone",
        "\u{1F64E}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_pouting_medium_skin_tone",
        "\u{1F64E}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_raising_hand", "\u{1F64B}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_raising_hand_dark_skin_tone",
        "\u{1F64B}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_raising_hand_light_skin_tone",
        "\u{1F64B}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_raising_hand_medium-dark_skin_tone",
        "\u{1F64B}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_raising_hand_medium-light_skin_tone",
        "\u{1F64B}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_raising_hand_medium_skin_tone",
        "\u{1F64B}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_rowing_boat", "\u{1F6A3}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_rowing_boat_dark_skin_tone",
        "\u{1F6A3}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_rowing_boat_light_skin_tone",
        "\u{1F6A3}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_rowing_boat_medium-dark_skin_tone",
        "\u{1F6A3}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_rowing_boat_medium-light_skin_tone",
        "\u{1F6A3}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_rowing_boat_medium_skin_tone",
        "\u{1F6A3}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_running", "\u{1F3C3}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_running_dark_skin_tone",
        "\u{1F3C3}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_running_light_skin_tone",
        "\u{1F3C3}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_running_medium-dark_skin_tone",
        "\u{1F3C3}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_running_medium-light_skin_tone",
        "\u{1F3C3}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_running_medium_skin_tone",
        "\u{1F3C3}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_scientist", "\u{1F468}\u{200D}\u{1F52C}");
    m.insert(
        "man_scientist_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F52C}",
    );
    m.insert(
        "man_scientist_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F52C}",
    );
    m.insert(
        "man_scientist_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F52C}",
    );
    m.insert(
        "man_scientist_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F52C}",
    );
    m.insert(
        "man_scientist_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F52C}",
    );
    m.insert("man_shrugging", "\u{1F937}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_shrugging_dark_skin_tone",
        "\u{1F937}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_shrugging_light_skin_tone",
        "\u{1F937}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_shrugging_medium-dark_skin_tone",
        "\u{1F937}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_shrugging_medium-light_skin_tone",
        "\u{1F937}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_shrugging_medium_skin_tone",
        "\u{1F937}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_singer", "\u{1F468}\u{200D}\u{1F3A4}");
    m.insert(
        "man_singer_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "man_singer_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "man_singer_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "man_singer_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "man_singer_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F3A4}",
    );
    m.insert("man_student", "\u{1F468}\u{200D}\u{1F393}");
    m.insert(
        "man_student_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F393}",
    );
    m.insert(
        "man_student_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F393}",
    );
    m.insert(
        "man_student_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F393}",
    );
    m.insert(
        "man_student_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F393}",
    );
    m.insert(
        "man_student_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F393}",
    );
    m.insert("man_surfing", "\u{1F3C4}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_surfing_dark_skin_tone",
        "\u{1F3C4}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_surfing_light_skin_tone",
        "\u{1F3C4}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_surfing_medium-dark_skin_tone",
        "\u{1F3C4}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_surfing_medium-light_skin_tone",
        "\u{1F3C4}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_surfing_medium_skin_tone",
        "\u{1F3C4}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_swimming", "\u{1F3CA}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_swimming_dark_skin_tone",
        "\u{1F3CA}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_swimming_light_skin_tone",
        "\u{1F3CA}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_swimming_medium-dark_skin_tone",
        "\u{1F3CA}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_swimming_medium-light_skin_tone",
        "\u{1F3CA}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_swimming_medium_skin_tone",
        "\u{1F3CA}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_teacher", "\u{1F468}\u{200D}\u{1F3EB}");
    m.insert(
        "man_teacher_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "man_teacher_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "man_teacher_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "man_teacher_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "man_teacher_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F3EB}",
    );
    m.insert("man_technologist", "\u{1F468}\u{200D}\u{1F4BB}");
    m.insert(
        "man_technologist_dark_skin_tone",
        "\u{1F468}\u{1F3FF}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "man_technologist_light_skin_tone",
        "\u{1F468}\u{1F3FB}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "man_technologist_medium-dark_skin_tone",
        "\u{1F468}\u{1F3FE}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "man_technologist_medium-light_skin_tone",
        "\u{1F468}\u{1F3FC}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "man_technologist_medium_skin_tone",
        "\u{1F468}\u{1F3FD}\u{200D}\u{1F4BB}",
    );
    m.insert("man_tipping_hand", "\u{1F481}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_tipping_hand_dark_skin_tone",
        "\u{1F481}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_tipping_hand_light_skin_tone",
        "\u{1F481}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_tipping_hand_medium-dark_skin_tone",
        "\u{1F481}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_tipping_hand_medium-light_skin_tone",
        "\u{1F481}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_tipping_hand_medium_skin_tone",
        "\u{1F481}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_vampire", "\u{1F9DB}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_vampire_dark_skin_tone",
        "\u{1F9DB}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_vampire_light_skin_tone",
        "\u{1F9DB}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_vampire_medium-dark_skin_tone",
        "\u{1F9DB}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_vampire_medium-light_skin_tone",
        "\u{1F9DB}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_vampire_medium_skin_tone",
        "\u{1F9DB}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_walking", "\u{1F6B6}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_walking_dark_skin_tone",
        "\u{1F6B6}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_walking_light_skin_tone",
        "\u{1F6B6}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_walking_medium-dark_skin_tone",
        "\u{1F6B6}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_walking_medium-light_skin_tone",
        "\u{1F6B6}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_walking_medium_skin_tone",
        "\u{1F6B6}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_wearing_turban", "\u{1F473}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "man_wearing_turban_dark_skin_tone",
        "\u{1F473}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_wearing_turban_light_skin_tone",
        "\u{1F473}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_wearing_turban_medium-dark_skin_tone",
        "\u{1F473}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_wearing_turban_medium-light_skin_tone",
        "\u{1F473}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "man_wearing_turban_medium_skin_tone",
        "\u{1F473}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("man_with_probing_cane", "\u{1F468}\u{200D}\u{1F9AF}");
    m.insert("man_with_chinese_cap", "\u{1F472}");
    m.insert("man_with_chinese_cap_dark_skin_tone", "\u{1F472}\u{1F3FF}");
    m.insert("man_with_chinese_cap_light_skin_tone", "\u{1F472}\u{1F3FB}");
    m.insert(
        "man_with_chinese_cap_medium-dark_skin_tone",
        "\u{1F472}\u{1F3FE}",
    );
    m.insert(
        "man_with_chinese_cap_medium-light_skin_tone",
        "\u{1F472}\u{1F3FC}",
    );
    m.insert(
        "man_with_chinese_cap_medium_skin_tone",
        "\u{1F472}\u{1F3FD}",
    );
    m.insert("man_zombie", "\u{1F9DF}\u{200D}\u{2642}\u{FE0F}");
    m.insert("mango", "\u{1F96D}");
    m.insert("mantelpiece_clock", "\u{1F570}");
    m.insert("manual_wheelchair", "\u{1F9BD}");
    m.insert("man’s_shoe", "\u{1F45E}");
    m.insert("map_of_japan", "\u{1F5FE}");
    m.insert("maple_leaf", "\u{1F341}");
    m.insert("martial_arts_uniform", "\u{1F94B}");
    m.insert("mate", "\u{1F9C9}");
    m.insert("meat_on_bone", "\u{1F356}");
    m.insert("mechanical_arm", "\u{1F9BE}");
    m.insert("mechanical_leg", "\u{1F9BF}");
    m.insert("medical_symbol", "\u{2695}");
    m.insert("megaphone", "\u{1F4E3}");
    m.insert("melon", "\u{1F348}");
    m.insert("memo", "\u{1F4DD}");
    m.insert("men_with_bunny_ears", "\u{1F46F}\u{200D}\u{2642}\u{FE0F}");
    m.insert("men_wrestling", "\u{1F93C}\u{200D}\u{2642}\u{FE0F}");
    m.insert("menorah", "\u{1F54E}");
    m.insert("men’s_room", "\u{1F6B9}");
    m.insert("mermaid", "\u{1F9DC}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "mermaid_dark_skin_tone",
        "\u{1F9DC}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "mermaid_light_skin_tone",
        "\u{1F9DC}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "mermaid_medium-dark_skin_tone",
        "\u{1F9DC}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "mermaid_medium-light_skin_tone",
        "\u{1F9DC}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "mermaid_medium_skin_tone",
        "\u{1F9DC}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("merman", "\u{1F9DC}\u{200D}\u{2642}\u{FE0F}");
    m.insert(
        "merman_dark_skin_tone",
        "\u{1F9DC}\u{1F3FF}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "merman_light_skin_tone",
        "\u{1F9DC}\u{1F3FB}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "merman_medium-dark_skin_tone",
        "\u{1F9DC}\u{1F3FE}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "merman_medium-light_skin_tone",
        "\u{1F9DC}\u{1F3FC}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert(
        "merman_medium_skin_tone",
        "\u{1F9DC}\u{1F3FD}\u{200D}\u{2642}\u{FE0F}",
    );
    m.insert("merperson", "\u{1F9DC}");
    m.insert("merperson_dark_skin_tone", "\u{1F9DC}\u{1F3FF}");
    m.insert("merperson_light_skin_tone", "\u{1F9DC}\u{1F3FB}");
    m.insert("merperson_medium-dark_skin_tone", "\u{1F9DC}\u{1F3FE}");
    m.insert("merperson_medium-light_skin_tone", "\u{1F9DC}\u{1F3FC}");
    m.insert("merperson_medium_skin_tone", "\u{1F9DC}\u{1F3FD}");
    m.insert("metro", "\u{1F687}");
    m.insert("microbe", "\u{1F9A0}");
    m.insert("microphone", "\u{1F3A4}");
    m.insert("microscope", "\u{1F52C}");
    m.insert("middle_finger", "\u{1F595}");
    m.insert("middle_finger_dark_skin_tone", "\u{1F595}\u{1F3FF}");
    m.insert("middle_finger_light_skin_tone", "\u{1F595}\u{1F3FB}");
    m.insert("middle_finger_medium-dark_skin_tone", "\u{1F595}\u{1F3FE}");
    m.insert("middle_finger_medium-light_skin_tone", "\u{1F595}\u{1F3FC}");
    m.insert("middle_finger_medium_skin_tone", "\u{1F595}\u{1F3FD}");
    m.insert("military_medal", "\u{1F396}");
    m.insert("milky_way", "\u{1F30C}");
    m.insert("minibus", "\u{1F690}");
    m.insert("moai", "\u{1F5FF}");
    m.insert("mobile_phone", "\u{1F4F1}");
    m.insert("mobile_phone_off", "\u{1F4F4}");
    m.insert("mobile_phone_with_arrow", "\u{1F4F2}");
    m.insert("money-mouth_face", "\u{1F911}");
    m.insert("money_bag", "\u{1F4B0}");
    m.insert("money_with_wings", "\u{1F4B8}");
    m.insert("monkey", "\u{1F412}");
    m.insert("monkey_face", "\u{1F435}");
    m.insert("monorail", "\u{1F69D}");
    m.insert("moon_cake", "\u{1F96E}");
    m.insert("moon_viewing_ceremony", "\u{1F391}");
    m.insert("mosque", "\u{1F54C}");
    m.insert("mosquito", "\u{1F99F}");
    m.insert("motor_boat", "\u{1F6E5}");
    m.insert("motor_scooter", "\u{1F6F5}");
    m.insert("motorcycle", "\u{1F3CD}");
    m.insert("motorized_wheelchair", "\u{1F9BC}");
    m.insert("motorway", "\u{1F6E3}");
    m.insert("mount_fuji", "\u{1F5FB}");
    m.insert("mountain", "\u{26F0}");
    m.insert("mountain_cableway", "\u{1F6A0}");
    m.insert("mountain_railway", "\u{1F69E}");
    m.insert("mouse", "\u{1F42D}");
    m.insert("mouse_face", "\u{1F42D}");
    m.insert("mouth", "\u{1F444}");
    m.insert("movie_camera", "\u{1F3A5}");
    m.insert("mushroom", "\u{1F344}");
    m.insert("musical_keyboard", "\u{1F3B9}");
    m.insert("musical_note", "\u{1F3B5}");
    m.insert("musical_notes", "\u{1F3B6}");
    m.insert("musical_score", "\u{1F3BC}");
    m.insert("muted_speaker", "\u{1F507}");
    m.insert("nail_polish", "\u{1F485}");
    m.insert("nail_polish_dark_skin_tone", "\u{1F485}\u{1F3FF}");
    m.insert("nail_polish_light_skin_tone", "\u{1F485}\u{1F3FB}");
    m.insert("nail_polish_medium-dark_skin_tone", "\u{1F485}\u{1F3FE}");
    m.insert("nail_polish_medium-light_skin_tone", "\u{1F485}\u{1F3FC}");
    m.insert("nail_polish_medium_skin_tone", "\u{1F485}\u{1F3FD}");
    m.insert("name_badge", "\u{1F4DB}");
    m.insert("national_park", "\u{1F3DE}");
    m.insert("nauseated_face", "\u{1F922}");
    m.insert("nazar_amulet", "\u{1F9FF}");
    m.insert("necktie", "\u{1F454}");
    m.insert("nerd_face", "\u{1F913}");
    m.insert("neutral_face", "\u{1F610}");
    m.insert("new_moon", "\u{1F311}");
    m.insert("new_moon_face", "\u{1F31A}");
    m.insert("newspaper", "\u{1F4F0}");
    m.insert("next_track_button", "\u{23ED}");
    m.insert("night_with_stars", "\u{1F303}");
    m.insert("nine-thirty", "\u{1F564}");
    m.insert("nine_o’clock", "\u{1F558}");
    m.insert("no_bicycles", "\u{1F6B3}");
    m.insert("no_entry", "\u{26D4}");
    m.insert("no_littering", "\u{1F6AF}");
    m.insert("no_mobile_phones", "\u{1F4F5}");
    m.insert("no_one_under_eighteen", "\u{1F51E}");
    m.insert("no_pedestrians", "\u{1F6B7}");
    m.insert("no_smoking", "\u{1F6AD}");
    m.insert("non-potable_water", "\u{1F6B1}");
    m.insert("nose", "\u{1F443}");
    m.insert("nose_dark_skin_tone", "\u{1F443}\u{1F3FF}");
    m.insert("nose_light_skin_tone", "\u{1F443}\u{1F3FB}");
    m.insert("nose_medium-dark_skin_tone", "\u{1F443}\u{1F3FE}");
    m.insert("nose_medium-light_skin_tone", "\u{1F443}\u{1F3FC}");
    m.insert("nose_medium_skin_tone", "\u{1F443}\u{1F3FD}");
    m.insert("notebook", "\u{1F4D3}");
    m.insert("notebook_with_decorative_cover", "\u{1F4D4}");
    m.insert("nut_and_bolt", "\u{1F529}");
    m.insert("octopus", "\u{1F419}");
    m.insert("oden", "\u{1F362}");
    m.insert("office_building", "\u{1F3E2}");
    m.insert("ogre", "\u{1F479}");
    m.insert("oil_drum", "\u{1F6E2}");
    m.insert("old_key", "\u{1F5DD}");
    m.insert("old_man", "\u{1F474}");
    m.insert("old_man_dark_skin_tone", "\u{1F474}\u{1F3FF}");
    m.insert("old_man_light_skin_tone", "\u{1F474}\u{1F3FB}");
    m.insert("old_man_medium-dark_skin_tone", "\u{1F474}\u{1F3FE}");
    m.insert("old_man_medium-light_skin_tone", "\u{1F474}\u{1F3FC}");
    m.insert("old_man_medium_skin_tone", "\u{1F474}\u{1F3FD}");
    m.insert("old_woman", "\u{1F475}");
    m.insert("old_woman_dark_skin_tone", "\u{1F475}\u{1F3FF}");
    m.insert("old_woman_light_skin_tone", "\u{1F475}\u{1F3FB}");
    m.insert("old_woman_medium-dark_skin_tone", "\u{1F475}\u{1F3FE}");
    m.insert("old_woman_medium-light_skin_tone", "\u{1F475}\u{1F3FC}");
    m.insert("old_woman_medium_skin_tone", "\u{1F475}\u{1F3FD}");
    m.insert("older_adult", "\u{1F9D3}");
    m.insert("older_adult_dark_skin_tone", "\u{1F9D3}\u{1F3FF}");
    m.insert("older_adult_light_skin_tone", "\u{1F9D3}\u{1F3FB}");
    m.insert("older_adult_medium-dark_skin_tone", "\u{1F9D3}\u{1F3FE}");
    m.insert("older_adult_medium-light_skin_tone", "\u{1F9D3}\u{1F3FC}");
    m.insert("older_adult_medium_skin_tone", "\u{1F9D3}\u{1F3FD}");
    m.insert("om", "\u{1F549}");
    m.insert("oncoming_automobile", "\u{1F698}");
    m.insert("oncoming_bus", "\u{1F68D}");
    m.insert("oncoming_fist", "\u{1F44A}");
    m.insert("oncoming_fist_dark_skin_tone", "\u{1F44A}\u{1F3FF}");
    m.insert("oncoming_fist_light_skin_tone", "\u{1F44A}\u{1F3FB}");
    m.insert("oncoming_fist_medium-dark_skin_tone", "\u{1F44A}\u{1F3FE}");
    m.insert("oncoming_fist_medium-light_skin_tone", "\u{1F44A}\u{1F3FC}");
    m.insert("oncoming_fist_medium_skin_tone", "\u{1F44A}\u{1F3FD}");
    m.insert("oncoming_police_car", "\u{1F694}");
    m.insert("oncoming_taxi", "\u{1F696}");
    m.insert("one-piece_swimsuit", "\u{1FA71}");
    m.insert("one-thirty", "\u{1F55C}");
    m.insert("one_o’clock", "\u{1F550}");
    m.insert("onion", "\u{1F9C5}");
    m.insert("open_book", "\u{1F4D6}");
    m.insert("open_file_folder", "\u{1F4C2}");
    m.insert("open_hands", "\u{1F450}");
    m.insert("open_hands_dark_skin_tone", "\u{1F450}\u{1F3FF}");
    m.insert("open_hands_light_skin_tone", "\u{1F450}\u{1F3FB}");
    m.insert("open_hands_medium-dark_skin_tone", "\u{1F450}\u{1F3FE}");
    m.insert("open_hands_medium-light_skin_tone", "\u{1F450}\u{1F3FC}");
    m.insert("open_hands_medium_skin_tone", "\u{1F450}\u{1F3FD}");
    m.insert("open_mailbox_with_lowered_flag", "\u{1F4ED}");
    m.insert("open_mailbox_with_raised_flag", "\u{1F4EC}");
    m.insert("optical_disk", "\u{1F4BF}");
    m.insert("orange_book", "\u{1F4D9}");
    m.insert("orange_circle", "\u{1F7E0}");
    m.insert("orange_heart", "\u{1F9E1}");
    m.insert("orange_square", "\u{1F7E7}");
    m.insert("orangutan", "\u{1F9A7}");
    m.insert("orthodox_cross", "\u{2626}");
    m.insert("otter", "\u{1F9A6}");
    m.insert("outbox_tray", "\u{1F4E4}");
    m.insert("owl", "\u{1F989}");
    m.insert("ox", "\u{1F402}");
    m.insert("oyster", "\u{1F9AA}");
    m.insert("package", "\u{1F4E6}");
    m.insert("page_facing_up", "\u{1F4C4}");
    m.insert("page_with_curl", "\u{1F4C3}");
    m.insert("pager", "\u{1F4DF}");
    m.insert("paintbrush", "\u{1F58C}");
    m.insert("palm_tree", "\u{1F334}");
    m.insert("palms_up_together", "\u{1F932}");
    m.insert("palms_up_together_dark_skin_tone", "\u{1F932}\u{1F3FF}");
    m.insert("palms_up_together_light_skin_tone", "\u{1F932}\u{1F3FB}");
    m.insert(
        "palms_up_together_medium-dark_skin_tone",
        "\u{1F932}\u{1F3FE}",
    );
    m.insert(
        "palms_up_together_medium-light_skin_tone",
        "\u{1F932}\u{1F3FC}",
    );
    m.insert("palms_up_together_medium_skin_tone", "\u{1F932}\u{1F3FD}");
    m.insert("pancakes", "\u{1F95E}");
    m.insert("panda_face", "\u{1F43C}");
    m.insert("paperclip", "\u{1F4CE}");
    m.insert("parrot", "\u{1F99C}");
    m.insert("part_alternation_mark", "\u{303D}");
    m.insert("party_popper", "\u{1F389}");
    m.insert("partying_face", "\u{1F973}");
    m.insert("passenger_ship", "\u{1F6F3}");
    m.insert("passport_control", "\u{1F6C2}");
    m.insert("pause_button", "\u{23F8}");
    m.insert("paw_prints", "\u{1F43E}");
    m.insert("peace_symbol", "\u{262E}");
    m.insert("peach", "\u{1F351}");
    m.insert("peacock", "\u{1F99A}");
    m.insert("peanuts", "\u{1F95C}");
    m.insert("pear", "\u{1F350}");
    m.insert("pen", "\u{1F58A}");
    m.insert("pencil", "\u{1F4DD}");
    m.insert("penguin", "\u{1F427}");
    m.insert("pensive_face", "\u{1F614}");
    m.insert(
        "people_holding_hands",
        "\u{1F9D1}\u{200D}\u{1F91D}\u{200D}\u{1F9D1}",
    );
    m.insert("people_with_bunny_ears", "\u{1F46F}");
    m.insert("people_wrestling", "\u{1F93C}");
    m.insert("performing_arts", "\u{1F3AD}");
    m.insert("persevering_face", "\u{1F623}");
    m.insert("person_biking", "\u{1F6B4}");
    m.insert("person_biking_dark_skin_tone", "\u{1F6B4}\u{1F3FF}");
    m.insert("person_biking_light_skin_tone", "\u{1F6B4}\u{1F3FB}");
    m.insert("person_biking_medium-dark_skin_tone", "\u{1F6B4}\u{1F3FE}");
    m.insert("person_biking_medium-light_skin_tone", "\u{1F6B4}\u{1F3FC}");
    m.insert("person_biking_medium_skin_tone", "\u{1F6B4}\u{1F3FD}");
    m.insert("person_bouncing_ball", "\u{26F9}");
    m.insert("person_bouncing_ball_dark_skin_tone", "\u{26F9}\u{1F3FF}");
    m.insert("person_bouncing_ball_light_skin_tone", "\u{26F9}\u{1F3FB}");
    m.insert(
        "person_bouncing_ball_medium-dark_skin_tone",
        "\u{26F9}\u{1F3FE}",
    );
    m.insert(
        "person_bouncing_ball_medium-light_skin_tone",
        "\u{26F9}\u{1F3FC}",
    );
    m.insert("person_bouncing_ball_medium_skin_tone", "\u{26F9}\u{1F3FD}");
    m.insert("person_bowing", "\u{1F647}");
    m.insert("person_bowing_dark_skin_tone", "\u{1F647}\u{1F3FF}");
    m.insert("person_bowing_light_skin_tone", "\u{1F647}\u{1F3FB}");
    m.insert("person_bowing_medium-dark_skin_tone", "\u{1F647}\u{1F3FE}");
    m.insert("person_bowing_medium-light_skin_tone", "\u{1F647}\u{1F3FC}");
    m.insert("person_bowing_medium_skin_tone", "\u{1F647}\u{1F3FD}");
    m.insert("person_cartwheeling", "\u{1F938}");
    m.insert("person_cartwheeling_dark_skin_tone", "\u{1F938}\u{1F3FF}");
    m.insert("person_cartwheeling_light_skin_tone", "\u{1F938}\u{1F3FB}");
    m.insert(
        "person_cartwheeling_medium-dark_skin_tone",
        "\u{1F938}\u{1F3FE}",
    );
    m.insert(
        "person_cartwheeling_medium-light_skin_tone",
        "\u{1F938}\u{1F3FC}",
    );
    m.insert("person_cartwheeling_medium_skin_tone", "\u{1F938}\u{1F3FD}");
    m.insert("person_climbing", "\u{1F9D7}");
    m.insert("person_climbing_dark_skin_tone", "\u{1F9D7}\u{1F3FF}");
    m.insert("person_climbing_light_skin_tone", "\u{1F9D7}\u{1F3FB}");
    m.insert(
        "person_climbing_medium-dark_skin_tone",
        "\u{1F9D7}\u{1F3FE}",
    );
    m.insert(
        "person_climbing_medium-light_skin_tone",
        "\u{1F9D7}\u{1F3FC}",
    );
    m.insert("person_climbing_medium_skin_tone", "\u{1F9D7}\u{1F3FD}");
    m.insert("person_facepalming", "\u{1F926}");
    m.insert("person_facepalming_dark_skin_tone", "\u{1F926}\u{1F3FF}");
    m.insert("person_facepalming_light_skin_tone", "\u{1F926}\u{1F3FB}");
    m.insert(
        "person_facepalming_medium-dark_skin_tone",
        "\u{1F926}\u{1F3FE}",
    );
    m.insert(
        "person_facepalming_medium-light_skin_tone",
        "\u{1F926}\u{1F3FC}",
    );
    m.insert("person_facepalming_medium_skin_tone", "\u{1F926}\u{1F3FD}");
    m.insert("person_fencing", "\u{1F93A}");
    m.insert("person_frowning", "\u{1F64D}");
    m.insert("person_frowning_dark_skin_tone", "\u{1F64D}\u{1F3FF}");
    m.insert("person_frowning_light_skin_tone", "\u{1F64D}\u{1F3FB}");
    m.insert(
        "person_frowning_medium-dark_skin_tone",
        "\u{1F64D}\u{1F3FE}",
    );
    m.insert(
        "person_frowning_medium-light_skin_tone",
        "\u{1F64D}\u{1F3FC}",
    );
    m.insert("person_frowning_medium_skin_tone", "\u{1F64D}\u{1F3FD}");
    m.insert("person_gesturing_no", "\u{1F645}");
    m.insert("person_gesturing_no_dark_skin_tone", "\u{1F645}\u{1F3FF}");
    m.insert("person_gesturing_no_light_skin_tone", "\u{1F645}\u{1F3FB}");
    m.insert(
        "person_gesturing_no_medium-dark_skin_tone",
        "\u{1F645}\u{1F3FE}",
    );
    m.insert(
        "person_gesturing_no_medium-light_skin_tone",
        "\u{1F645}\u{1F3FC}",
    );
    m.insert("person_gesturing_no_medium_skin_tone", "\u{1F645}\u{1F3FD}");
    m.insert("person_gesturing_ok", "\u{1F646}");
    m.insert("person_gesturing_ok_dark_skin_tone", "\u{1F646}\u{1F3FF}");
    m.insert("person_gesturing_ok_light_skin_tone", "\u{1F646}\u{1F3FB}");
    m.insert(
        "person_gesturing_ok_medium-dark_skin_tone",
        "\u{1F646}\u{1F3FE}",
    );
    m.insert(
        "person_gesturing_ok_medium-light_skin_tone",
        "\u{1F646}\u{1F3FC}",
    );
    m.insert("person_gesturing_ok_medium_skin_tone", "\u{1F646}\u{1F3FD}");
    m.insert("person_getting_haircut", "\u{1F487}");
    m.insert(
        "person_getting_haircut_dark_skin_tone",
        "\u{1F487}\u{1F3FF}",
    );
    m.insert(
        "person_getting_haircut_light_skin_tone",
        "\u{1F487}\u{1F3FB}",
    );
    m.insert(
        "person_getting_haircut_medium-dark_skin_tone",
        "\u{1F487}\u{1F3FE}",
    );
    m.insert(
        "person_getting_haircut_medium-light_skin_tone",
        "\u{1F487}\u{1F3FC}",
    );
    m.insert(
        "person_getting_haircut_medium_skin_tone",
        "\u{1F487}\u{1F3FD}",
    );
    m.insert("person_getting_massage", "\u{1F486}");
    m.insert(
        "person_getting_massage_dark_skin_tone",
        "\u{1F486}\u{1F3FF}",
    );
    m.insert(
        "person_getting_massage_light_skin_tone",
        "\u{1F486}\u{1F3FB}",
    );
    m.insert(
        "person_getting_massage_medium-dark_skin_tone",
        "\u{1F486}\u{1F3FE}",
    );
    m.insert(
        "person_getting_massage_medium-light_skin_tone",
        "\u{1F486}\u{1F3FC}",
    );
    m.insert(
        "person_getting_massage_medium_skin_tone",
        "\u{1F486}\u{1F3FD}",
    );
    m.insert("person_golfing", "\u{1F3CC}");
    m.insert("person_golfing_dark_skin_tone", "\u{1F3CC}\u{1F3FF}");
    m.insert("person_golfing_light_skin_tone", "\u{1F3CC}\u{1F3FB}");
    m.insert("person_golfing_medium-dark_skin_tone", "\u{1F3CC}\u{1F3FE}");
    m.insert(
        "person_golfing_medium-light_skin_tone",
        "\u{1F3CC}\u{1F3FC}",
    );
    m.insert("person_golfing_medium_skin_tone", "\u{1F3CC}\u{1F3FD}");
    m.insert("person_in_bed", "\u{1F6CC}");
    m.insert("person_in_bed_dark_skin_tone", "\u{1F6CC}\u{1F3FF}");
    m.insert("person_in_bed_light_skin_tone", "\u{1F6CC}\u{1F3FB}");
    m.insert("person_in_bed_medium-dark_skin_tone", "\u{1F6CC}\u{1F3FE}");
    m.insert("person_in_bed_medium-light_skin_tone", "\u{1F6CC}\u{1F3FC}");
    m.insert("person_in_bed_medium_skin_tone", "\u{1F6CC}\u{1F3FD}");
    m.insert("person_in_lotus_position", "\u{1F9D8}");
    m.insert(
        "person_in_lotus_position_dark_skin_tone",
        "\u{1F9D8}\u{1F3FF}",
    );
    m.insert(
        "person_in_lotus_position_light_skin_tone",
        "\u{1F9D8}\u{1F3FB}",
    );
    m.insert(
        "person_in_lotus_position_medium-dark_skin_tone",
        "\u{1F9D8}\u{1F3FE}",
    );
    m.insert(
        "person_in_lotus_position_medium-light_skin_tone",
        "\u{1F9D8}\u{1F3FC}",
    );
    m.insert(
        "person_in_lotus_position_medium_skin_tone",
        "\u{1F9D8}\u{1F3FD}",
    );
    m.insert("person_in_steamy_room", "\u{1F9D6}");
    m.insert("person_in_steamy_room_dark_skin_tone", "\u{1F9D6}\u{1F3FF}");
    m.insert(
        "person_in_steamy_room_light_skin_tone",
        "\u{1F9D6}\u{1F3FB}",
    );
    m.insert(
        "person_in_steamy_room_medium-dark_skin_tone",
        "\u{1F9D6}\u{1F3FE}",
    );
    m.insert(
        "person_in_steamy_room_medium-light_skin_tone",
        "\u{1F9D6}\u{1F3FC}",
    );
    m.insert(
        "person_in_steamy_room_medium_skin_tone",
        "\u{1F9D6}\u{1F3FD}",
    );
    m.insert("person_juggling", "\u{1F939}");
    m.insert("person_juggling_dark_skin_tone", "\u{1F939}\u{1F3FF}");
    m.insert("person_juggling_light_skin_tone", "\u{1F939}\u{1F3FB}");
    m.insert(
        "person_juggling_medium-dark_skin_tone",
        "\u{1F939}\u{1F3FE}",
    );
    m.insert(
        "person_juggling_medium-light_skin_tone",
        "\u{1F939}\u{1F3FC}",
    );
    m.insert("person_juggling_medium_skin_tone", "\u{1F939}\u{1F3FD}");
    m.insert("person_kneeling", "\u{1F9CE}");
    m.insert("person_lifting_weights", "\u{1F3CB}");
    m.insert(
        "person_lifting_weights_dark_skin_tone",
        "\u{1F3CB}\u{1F3FF}",
    );
    m.insert(
        "person_lifting_weights_light_skin_tone",
        "\u{1F3CB}\u{1F3FB}",
    );
    m.insert(
        "person_lifting_weights_medium-dark_skin_tone",
        "\u{1F3CB}\u{1F3FE}",
    );
    m.insert(
        "person_lifting_weights_medium-light_skin_tone",
        "\u{1F3CB}\u{1F3FC}",
    );
    m.insert(
        "person_lifting_weights_medium_skin_tone",
        "\u{1F3CB}\u{1F3FD}",
    );
    m.insert("person_mountain_biking", "\u{1F6B5}");
    m.insert(
        "person_mountain_biking_dark_skin_tone",
        "\u{1F6B5}\u{1F3FF}",
    );
    m.insert(
        "person_mountain_biking_light_skin_tone",
        "\u{1F6B5}\u{1F3FB}",
    );
    m.insert(
        "person_mountain_biking_medium-dark_skin_tone",
        "\u{1F6B5}\u{1F3FE}",
    );
    m.insert(
        "person_mountain_biking_medium-light_skin_tone",
        "\u{1F6B5}\u{1F3FC}",
    );
    m.insert(
        "person_mountain_biking_medium_skin_tone",
        "\u{1F6B5}\u{1F3FD}",
    );
    m.insert("person_playing_handball", "\u{1F93E}");
    m.insert(
        "person_playing_handball_dark_skin_tone",
        "\u{1F93E}\u{1F3FF}",
    );
    m.insert(
        "person_playing_handball_light_skin_tone",
        "\u{1F93E}\u{1F3FB}",
    );
    m.insert(
        "person_playing_handball_medium-dark_skin_tone",
        "\u{1F93E}\u{1F3FE}",
    );
    m.insert(
        "person_playing_handball_medium-light_skin_tone",
        "\u{1F93E}\u{1F3FC}",
    );
    m.insert(
        "person_playing_handball_medium_skin_tone",
        "\u{1F93E}\u{1F3FD}",
    );
    m.insert("person_playing_water_polo", "\u{1F93D}");
    m.insert(
        "person_playing_water_polo_dark_skin_tone",
        "\u{1F93D}\u{1F3FF}",
    );
    m.insert(
        "person_playing_water_polo_light_skin_tone",
        "\u{1F93D}\u{1F3FB}",
    );
    m.insert(
        "person_playing_water_polo_medium-dark_skin_tone",
        "\u{1F93D}\u{1F3FE}",
    );
    m.insert(
        "person_playing_water_polo_medium-light_skin_tone",
        "\u{1F93D}\u{1F3FC}",
    );
    m.insert(
        "person_playing_water_polo_medium_skin_tone",
        "\u{1F93D}\u{1F3FD}",
    );
    m.insert("person_pouting", "\u{1F64E}");
    m.insert("person_pouting_dark_skin_tone", "\u{1F64E}\u{1F3FF}");
    m.insert("person_pouting_light_skin_tone", "\u{1F64E}\u{1F3FB}");
    m.insert("person_pouting_medium-dark_skin_tone", "\u{1F64E}\u{1F3FE}");
    m.insert(
        "person_pouting_medium-light_skin_tone",
        "\u{1F64E}\u{1F3FC}",
    );
    m.insert("person_pouting_medium_skin_tone", "\u{1F64E}\u{1F3FD}");
    m.insert("person_raising_hand", "\u{1F64B}");
    m.insert("person_raising_hand_dark_skin_tone", "\u{1F64B}\u{1F3FF}");
    m.insert("person_raising_hand_light_skin_tone", "\u{1F64B}\u{1F3FB}");
    m.insert(
        "person_raising_hand_medium-dark_skin_tone",
        "\u{1F64B}\u{1F3FE}",
    );
    m.insert(
        "person_raising_hand_medium-light_skin_tone",
        "\u{1F64B}\u{1F3FC}",
    );
    m.insert("person_raising_hand_medium_skin_tone", "\u{1F64B}\u{1F3FD}");
    m.insert("person_rowing_boat", "\u{1F6A3}");
    m.insert("person_rowing_boat_dark_skin_tone", "\u{1F6A3}\u{1F3FF}");
    m.insert("person_rowing_boat_light_skin_tone", "\u{1F6A3}\u{1F3FB}");
    m.insert(
        "person_rowing_boat_medium-dark_skin_tone",
        "\u{1F6A3}\u{1F3FE}",
    );
    m.insert(
        "person_rowing_boat_medium-light_skin_tone",
        "\u{1F6A3}\u{1F3FC}",
    );
    m.insert("person_rowing_boat_medium_skin_tone", "\u{1F6A3}\u{1F3FD}");
    m.insert("person_running", "\u{1F3C3}");
    m.insert("person_running_dark_skin_tone", "\u{1F3C3}\u{1F3FF}");
    m.insert("person_running_light_skin_tone", "\u{1F3C3}\u{1F3FB}");
    m.insert("person_running_medium-dark_skin_tone", "\u{1F3C3}\u{1F3FE}");
    m.insert(
        "person_running_medium-light_skin_tone",
        "\u{1F3C3}\u{1F3FC}",
    );
    m.insert("person_running_medium_skin_tone", "\u{1F3C3}\u{1F3FD}");
    m.insert("person_shrugging", "\u{1F937}");
    m.insert("person_shrugging_dark_skin_tone", "\u{1F937}\u{1F3FF}");
    m.insert("person_shrugging_light_skin_tone", "\u{1F937}\u{1F3FB}");
    m.insert(
        "person_shrugging_medium-dark_skin_tone",
        "\u{1F937}\u{1F3FE}",
    );
    m.insert(
        "person_shrugging_medium-light_skin_tone",
        "\u{1F937}\u{1F3FC}",
    );
    m.insert("person_shrugging_medium_skin_tone", "\u{1F937}\u{1F3FD}");
    m.insert("person_standing", "\u{1F9CD}");
    m.insert("person_surfing", "\u{1F3C4}");
    m.insert("person_surfing_dark_skin_tone", "\u{1F3C4}\u{1F3FF}");
    m.insert("person_surfing_light_skin_tone", "\u{1F3C4}\u{1F3FB}");
    m.insert("person_surfing_medium-dark_skin_tone", "\u{1F3C4}\u{1F3FE}");
    m.insert(
        "person_surfing_medium-light_skin_tone",
        "\u{1F3C4}\u{1F3FC}",
    );
    m.insert("person_surfing_medium_skin_tone", "\u{1F3C4}\u{1F3FD}");
    m.insert("person_swimming", "\u{1F3CA}");
    m.insert("person_swimming_dark_skin_tone", "\u{1F3CA}\u{1F3FF}");
    m.insert("person_swimming_light_skin_tone", "\u{1F3CA}\u{1F3FB}");
    m.insert(
        "person_swimming_medium-dark_skin_tone",
        "\u{1F3CA}\u{1F3FE}",
    );
    m.insert(
        "person_swimming_medium-light_skin_tone",
        "\u{1F3CA}\u{1F3FC}",
    );
    m.insert("person_swimming_medium_skin_tone", "\u{1F3CA}\u{1F3FD}");
    m.insert("person_taking_bath", "\u{1F6C0}");
    m.insert("person_taking_bath_dark_skin_tone", "\u{1F6C0}\u{1F3FF}");
    m.insert("person_taking_bath_light_skin_tone", "\u{1F6C0}\u{1F3FB}");
    m.insert(
        "person_taking_bath_medium-dark_skin_tone",
        "\u{1F6C0}\u{1F3FE}",
    );
    m.insert(
        "person_taking_bath_medium-light_skin_tone",
        "\u{1F6C0}\u{1F3FC}",
    );
    m.insert("person_taking_bath_medium_skin_tone", "\u{1F6C0}\u{1F3FD}");
    m.insert("person_tipping_hand", "\u{1F481}");
    m.insert("person_tipping_hand_dark_skin_tone", "\u{1F481}\u{1F3FF}");
    m.insert("person_tipping_hand_light_skin_tone", "\u{1F481}\u{1F3FB}");
    m.insert(
        "person_tipping_hand_medium-dark_skin_tone",
        "\u{1F481}\u{1F3FE}",
    );
    m.insert(
        "person_tipping_hand_medium-light_skin_tone",
        "\u{1F481}\u{1F3FC}",
    );
    m.insert("person_tipping_hand_medium_skin_tone", "\u{1F481}\u{1F3FD}");
    m.insert("person_walking", "\u{1F6B6}");
    m.insert("person_walking_dark_skin_tone", "\u{1F6B6}\u{1F3FF}");
    m.insert("person_walking_light_skin_tone", "\u{1F6B6}\u{1F3FB}");
    m.insert("person_walking_medium-dark_skin_tone", "\u{1F6B6}\u{1F3FE}");
    m.insert(
        "person_walking_medium-light_skin_tone",
        "\u{1F6B6}\u{1F3FC}",
    );
    m.insert("person_walking_medium_skin_tone", "\u{1F6B6}\u{1F3FD}");
    m.insert("person_wearing_turban", "\u{1F473}");
    m.insert("person_wearing_turban_dark_skin_tone", "\u{1F473}\u{1F3FF}");
    m.insert(
        "person_wearing_turban_light_skin_tone",
        "\u{1F473}\u{1F3FB}",
    );
    m.insert(
        "person_wearing_turban_medium-dark_skin_tone",
        "\u{1F473}\u{1F3FE}",
    );
    m.insert(
        "person_wearing_turban_medium-light_skin_tone",
        "\u{1F473}\u{1F3FC}",
    );
    m.insert(
        "person_wearing_turban_medium_skin_tone",
        "\u{1F473}\u{1F3FD}",
    );
    m.insert("petri_dish", "\u{1F9EB}");
    m.insert("pick", "\u{26CF}");
    m.insert("pie", "\u{1F967}");
    m.insert("pig", "\u{1F437}");
    m.insert("pig_face", "\u{1F437}");
    m.insert("pig_nose", "\u{1F43D}");
    m.insert("pile_of_poo", "\u{1F4A9}");
    m.insert("pill", "\u{1F48A}");
    m.insert("pinching_hand", "\u{1F90F}");
    m.insert("pine_decoration", "\u{1F38D}");
    m.insert("pineapple", "\u{1F34D}");
    m.insert("ping_pong", "\u{1F3D3}");
    m.insert("pirate_flag", "\u{1F3F4}\u{200D}\u{2620}\u{FE0F}");
    m.insert("pistol", "\u{1F52B}");
    m.insert("pizza", "\u{1F355}");
    m.insert("place_of_worship", "\u{1F6D0}");
    m.insert("play_button", "\u{25B6}");
    m.insert("play_or_pause_button", "\u{23EF}");
    m.insert("pleading_face", "\u{1F97A}");
    m.insert("police_car", "\u{1F693}");
    m.insert("police_car_light", "\u{1F6A8}");
    m.insert("police_officer", "\u{1F46E}");
    m.insert("police_officer_dark_skin_tone", "\u{1F46E}\u{1F3FF}");
    m.insert("police_officer_light_skin_tone", "\u{1F46E}\u{1F3FB}");
    m.insert("police_officer_medium-dark_skin_tone", "\u{1F46E}\u{1F3FE}");
    m.insert(
        "police_officer_medium-light_skin_tone",
        "\u{1F46E}\u{1F3FC}",
    );
    m.insert("police_officer_medium_skin_tone", "\u{1F46E}\u{1F3FD}");
    m.insert("poodle", "\u{1F429}");
    m.insert("pool_8_ball", "\u{1F3B1}");
    m.insert("popcorn", "\u{1F37F}");
    m.insert("post_office", "\u{1F3E3}");
    m.insert("postal_horn", "\u{1F4EF}");
    m.insert("postbox", "\u{1F4EE}");
    m.insert("pot_of_food", "\u{1F372}");
    m.insert("potable_water", "\u{1F6B0}");
    m.insert("potato", "\u{1F954}");
    m.insert("poultry_leg", "\u{1F357}");
    m.insert("pound_banknote", "\u{1F4B7}");
    m.insert("pouting_cat_face", "\u{1F63E}");
    m.insert("pouting_face", "\u{1F621}");
    m.insert("prayer_beads", "\u{1F4FF}");
    m.insert("pregnant_woman", "\u{1F930}");
    m.insert("pregnant_woman_dark_skin_tone", "\u{1F930}\u{1F3FF}");
    m.insert("pregnant_woman_light_skin_tone", "\u{1F930}\u{1F3FB}");
    m.insert("pregnant_woman_medium-dark_skin_tone", "\u{1F930}\u{1F3FE}");
    m.insert(
        "pregnant_woman_medium-light_skin_tone",
        "\u{1F930}\u{1F3FC}",
    );
    m.insert("pregnant_woman_medium_skin_tone", "\u{1F930}\u{1F3FD}");
    m.insert("pretzel", "\u{1F968}");
    m.insert("probing_cane", "\u{1F9AF}");
    m.insert("prince", "\u{1F934}");
    m.insert("prince_dark_skin_tone", "\u{1F934}\u{1F3FF}");
    m.insert("prince_light_skin_tone", "\u{1F934}\u{1F3FB}");
    m.insert("prince_medium-dark_skin_tone", "\u{1F934}\u{1F3FE}");
    m.insert("prince_medium-light_skin_tone", "\u{1F934}\u{1F3FC}");
    m.insert("prince_medium_skin_tone", "\u{1F934}\u{1F3FD}");
    m.insert("princess", "\u{1F478}");
    m.insert("princess_dark_skin_tone", "\u{1F478}\u{1F3FF}");
    m.insert("princess_light_skin_tone", "\u{1F478}\u{1F3FB}");
    m.insert("princess_medium-dark_skin_tone", "\u{1F478}\u{1F3FE}");
    m.insert("princess_medium-light_skin_tone", "\u{1F478}\u{1F3FC}");
    m.insert("princess_medium_skin_tone", "\u{1F478}\u{1F3FD}");
    m.insert("printer", "\u{1F5A8}");
    m.insert("prohibited", "\u{1F6AB}");
    m.insert("purple_circle", "\u{1F7E3}");
    m.insert("purple_heart", "\u{1F49C}");
    m.insert("purple_square", "\u{1F7EA}");
    m.insert("purse", "\u{1F45B}");
    m.insert("pushpin", "\u{1F4CC}");
    m.insert("question_mark", "\u{2753}");
    m.insert("rabbit", "\u{1F430}");
    m.insert("rabbit_face", "\u{1F430}");
    m.insert("raccoon", "\u{1F99D}");
    m.insert("racing_car", "\u{1F3CE}");
    m.insert("radio", "\u{1F4FB}");
    m.insert("radio_button", "\u{1F518}");
    m.insert("radioactive", "\u{2622}");
    m.insert("railway_car", "\u{1F683}");
    m.insert("railway_track", "\u{1F6E4}");
    m.insert("rainbow", "\u{1F308}");
    m.insert("rainbow_flag", "\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}");
    m.insert("raised_back_of_hand", "\u{1F91A}");
    m.insert("raised_back_of_hand_dark_skin_tone", "\u{1F91A}\u{1F3FF}");
    m.insert("raised_back_of_hand_light_skin_tone", "\u{1F91A}\u{1F3FB}");
    m.insert(
        "raised_back_of_hand_medium-dark_skin_tone",
        "\u{1F91A}\u{1F3FE}",
    );
    m.insert(
        "raised_back_of_hand_medium-light_skin_tone",
        "\u{1F91A}\u{1F3FC}",
    );
    m.insert("raised_back_of_hand_medium_skin_tone", "\u{1F91A}\u{1F3FD}");
    m.insert("raised_fist", "\u{270A}");
    m.insert("raised_fist_dark_skin_tone", "\u{270A}\u{1F3FF}");
    m.insert("raised_fist_light_skin_tone", "\u{270A}\u{1F3FB}");
    m.insert("raised_fist_medium-dark_skin_tone", "\u{270A}\u{1F3FE}");
    m.insert("raised_fist_medium-light_skin_tone", "\u{270A}\u{1F3FC}");
    m.insert("raised_fist_medium_skin_tone", "\u{270A}\u{1F3FD}");
    m.insert("raised_hand", "\u{270B}");
    m.insert("raised_hand_dark_skin_tone", "\u{270B}\u{1F3FF}");
    m.insert("raised_hand_light_skin_tone", "\u{270B}\u{1F3FB}");
    m.insert("raised_hand_medium-dark_skin_tone", "\u{270B}\u{1F3FE}");
    m.insert("raised_hand_medium-light_skin_tone", "\u{270B}\u{1F3FC}");
    m.insert("raised_hand_medium_skin_tone", "\u{270B}\u{1F3FD}");
    m.insert("raising_hands", "\u{1F64C}");
    m.insert("raising_hands_dark_skin_tone", "\u{1F64C}\u{1F3FF}");
    m.insert("raising_hands_light_skin_tone", "\u{1F64C}\u{1F3FB}");
    m.insert("raising_hands_medium-dark_skin_tone", "\u{1F64C}\u{1F3FE}");
    m.insert("raising_hands_medium-light_skin_tone", "\u{1F64C}\u{1F3FC}");
    m.insert("raising_hands_medium_skin_tone", "\u{1F64C}\u{1F3FD}");
    m.insert("ram", "\u{1F40F}");
    m.insert("rat", "\u{1F400}");
    m.insert("razor", "\u{1FA92}");
    m.insert("ringed_planet", "\u{1FA90}");
    m.insert("receipt", "\u{1F9FE}");
    m.insert("record_button", "\u{23FA}");
    m.insert("recycling_symbol", "\u{267B}");
    m.insert("red_apple", "\u{1F34E}");
    m.insert("red_circle", "\u{1F534}");
    m.insert("red_envelope", "\u{1F9E7}");
    m.insert("red_hair", "\u{1F9B0}");
    m.insert("red-haired_man", "\u{1F468}\u{200D}\u{1F9B0}");
    m.insert("red-haired_woman", "\u{1F469}\u{200D}\u{1F9B0}");
    m.insert("red_heart", "\u{2764}");
    m.insert("red_paper_lantern", "\u{1F3EE}");
    m.insert("red_square", "\u{1F7E5}");
    m.insert("red_triangle_pointed_down", "\u{1F53B}");
    m.insert("red_triangle_pointed_up", "\u{1F53A}");
    m.insert("registered", "\u{AE}");
    m.insert("relieved_face", "\u{1F60C}");
    m.insert("reminder_ribbon", "\u{1F397}");
    m.insert("repeat_button", "\u{1F501}");
    m.insert("repeat_single_button", "\u{1F502}");
    m.insert("rescue_worker’s_helmet", "\u{26D1}");
    m.insert("restroom", "\u{1F6BB}");
    m.insert("reverse_button", "\u{25C0}");
    m.insert("revolving_hearts", "\u{1F49E}");
    m.insert("rhinoceros", "\u{1F98F}");
    m.insert("ribbon", "\u{1F380}");
    m.insert("rice_ball", "\u{1F359}");
    m.insert("rice_cracker", "\u{1F358}");
    m.insert("right-facing_fist", "\u{1F91C}");
    m.insert("right-facing_fist_dark_skin_tone", "\u{1F91C}\u{1F3FF}");
    m.insert("right-facing_fist_light_skin_tone", "\u{1F91C}\u{1F3FB}");
    m.insert(
        "right-facing_fist_medium-dark_skin_tone",
        "\u{1F91C}\u{1F3FE}",
    );
    m.insert(
        "right-facing_fist_medium-light_skin_tone",
        "\u{1F91C}\u{1F3FC}",
    );
    m.insert("right-facing_fist_medium_skin_tone", "\u{1F91C}\u{1F3FD}");
    m.insert("right_anger_bubble", "\u{1F5EF}");
    m.insert("right_arrow", "\u{27A1}");
    m.insert("right_arrow_curving_down", "\u{2935}");
    m.insert("right_arrow_curving_left", "\u{21A9}");
    m.insert("right_arrow_curving_up", "\u{2934}");
    m.insert("ring", "\u{1F48D}");
    m.insert("roasted_sweet_potato", "\u{1F360}");
    m.insert("robot_face", "\u{1F916}");
    m.insert("rocket", "\u{1F680}");
    m.insert("roll_of_paper", "\u{1F9FB}");
    m.insert("rolled-up_newspaper", "\u{1F5DE}");
    m.insert("roller_coaster", "\u{1F3A2}");
    m.insert("rolling_on_the_floor_laughing", "\u{1F923}");
    m.insert("rooster", "\u{1F413}");
    m.insert("rose", "\u{1F339}");
    m.insert("rosette", "\u{1F3F5}");
    m.insert("round_pushpin", "\u{1F4CD}");
    m.insert("rugby_football", "\u{1F3C9}");
    m.insert("running_shirt", "\u{1F3BD}");
    m.insert("running_shoe", "\u{1F45F}");
    m.insert("sad_but_relieved_face", "\u{1F625}");
    m.insert("safety_pin", "\u{1F9F7}");
    m.insert("safety_vest", "\u{1F9BA}");
    m.insert("salt", "\u{1F9C2}");
    m.insert("sailboat", "\u{26F5}");
    m.insert("sake", "\u{1F376}");
    m.insert("sandwich", "\u{1F96A}");
    m.insert("sari", "\u{1F97B}");
    m.insert("satellite", "\u{1F4E1}");
    m.insert("satellite_antenna", "\u{1F4E1}");
    m.insert("sauropod", "\u{1F995}");
    m.insert("saxophone", "\u{1F3B7}");
    m.insert("scarf", "\u{1F9E3}");
    m.insert("school", "\u{1F3EB}");
    m.insert("school_backpack", "\u{1F392}");
    m.insert("scissors", "\u{2702}");
    m.insert("scorpion", "\u{1F982}");
    m.insert("scroll", "\u{1F4DC}");
    m.insert("seat", "\u{1F4BA}");
    m.insert("see-no-evil_monkey", "\u{1F648}");
    m.insert("seedling", "\u{1F331}");
    m.insert("selfie", "\u{1F933}");
    m.insert("selfie_dark_skin_tone", "\u{1F933}\u{1F3FF}");
    m.insert("selfie_light_skin_tone", "\u{1F933}\u{1F3FB}");
    m.insert("selfie_medium-dark_skin_tone", "\u{1F933}\u{1F3FE}");
    m.insert("selfie_medium-light_skin_tone", "\u{1F933}\u{1F3FC}");
    m.insert("selfie_medium_skin_tone", "\u{1F933}\u{1F3FD}");
    m.insert("service_dog", "\u{1F415}\u{200D}\u{1F9BA}");
    m.insert("seven-thirty", "\u{1F562}");
    m.insert("seven_o’clock", "\u{1F556}");
    m.insert("shallow_pan_of_food", "\u{1F958}");
    m.insert("shamrock", "\u{2618}");
    m.insert("shark", "\u{1F988}");
    m.insert("shaved_ice", "\u{1F367}");
    m.insert("sheaf_of_rice", "\u{1F33E}");
    m.insert("shield", "\u{1F6E1}");
    m.insert("shinto_shrine", "\u{26E9}");
    m.insert("ship", "\u{1F6A2}");
    m.insert("shooting_star", "\u{1F320}");
    m.insert("shopping_bags", "\u{1F6CD}");
    m.insert("shopping_cart", "\u{1F6D2}");
    m.insert("shortcake", "\u{1F370}");
    m.insert("shorts", "\u{1FA73}");
    m.insert("shower", "\u{1F6BF}");
    m.insert("shrimp", "\u{1F990}");
    m.insert("shuffle_tracks_button", "\u{1F500}");
    m.insert("shushing_face", "\u{1F92B}");
    m.insert("sign_of_the_horns", "\u{1F918}");
    m.insert("sign_of_the_horns_dark_skin_tone", "\u{1F918}\u{1F3FF}");
    m.insert("sign_of_the_horns_light_skin_tone", "\u{1F918}\u{1F3FB}");
    m.insert(
        "sign_of_the_horns_medium-dark_skin_tone",
        "\u{1F918}\u{1F3FE}",
    );
    m.insert(
        "sign_of_the_horns_medium-light_skin_tone",
        "\u{1F918}\u{1F3FC}",
    );
    m.insert("sign_of_the_horns_medium_skin_tone", "\u{1F918}\u{1F3FD}");
    m.insert("six-thirty", "\u{1F561}");
    m.insert("six_o’clock", "\u{1F555}");
    m.insert("skateboard", "\u{1F6F9}");
    m.insert("skier", "\u{26F7}");
    m.insert("skis", "\u{1F3BF}");
    m.insert("skull", "\u{1F480}");
    m.insert("skull_and_crossbones", "\u{2620}");
    m.insert("skunk", "\u{1F9A8}");
    m.insert("sled", "\u{1F6F7}");
    m.insert("sleeping_face", "\u{1F634}");
    m.insert("sleepy_face", "\u{1F62A}");
    m.insert("slightly_frowning_face", "\u{1F641}");
    m.insert("slightly_smiling_face", "\u{1F642}");
    m.insert("slot_machine", "\u{1F3B0}");
    m.insert("sloth", "\u{1F9A5}");
    m.insert("small_airplane", "\u{1F6E9}");
    m.insert("small_blue_diamond", "\u{1F539}");
    m.insert("small_orange_diamond", "\u{1F538}");
    m.insert("smiling_cat_face_with_heart-eyes", "\u{1F63B}");
    m.insert("smiling_face", "\u{263A}");
    m.insert("smiling_face_with_halo", "\u{1F607}");
    m.insert("smiling_face_with_3_hearts", "\u{1F970}");
    m.insert("smiling_face_with_heart-eyes", "\u{1F60D}");
    m.insert("smiling_face_with_horns", "\u{1F608}");
    m.insert("smiling_face_with_smiling_eyes", "\u{1F60A}");
    m.insert("smiling_face_with_sunglasses", "\u{1F60E}");
    m.insert("smirking_face", "\u{1F60F}");
    m.insert("snail", "\u{1F40C}");
    m.insert("snake", "\u{1F40D}");
    m.insert("sneezing_face", "\u{1F927}");
    m.insert("snow-capped_mountain", "\u{1F3D4}");
    m.insert("snowboarder", "\u{1F3C2}");
    m.insert("snowboarder_dark_skin_tone", "\u{1F3C2}\u{1F3FF}");
    m.insert("snowboarder_light_skin_tone", "\u{1F3C2}\u{1F3FB}");
    m.insert("snowboarder_medium-dark_skin_tone", "\u{1F3C2}\u{1F3FE}");
    m.insert("snowboarder_medium-light_skin_tone", "\u{1F3C2}\u{1F3FC}");
    m.insert("snowboarder_medium_skin_tone", "\u{1F3C2}\u{1F3FD}");
    m.insert("snowflake", "\u{2744}");
    m.insert("snowman", "\u{2603}");
    m.insert("snowman_without_snow", "\u{26C4}");
    m.insert("soap", "\u{1F9FC}");
    m.insert("soccer_ball", "\u{26BD}");
    m.insert("socks", "\u{1F9E6}");
    m.insert("softball", "\u{1F94E}");
    m.insert("soft_ice_cream", "\u{1F366}");
    m.insert("spade_suit", "\u{2660}");
    m.insert("spaghetti", "\u{1F35D}");
    m.insert("sparkle", "\u{2747}");
    m.insert("sparkler", "\u{1F387}");
    m.insert("sparkles", "\u{2728}");
    m.insert("sparkling_heart", "\u{1F496}");
    m.insert("speak-no-evil_monkey", "\u{1F64A}");
    m.insert("speaker_high_volume", "\u{1F50A}");
    m.insert("speaker_low_volume", "\u{1F508}");
    m.insert("speaker_medium_volume", "\u{1F509}");
    m.insert("speaking_head", "\u{1F5E3}");
    m.insert("speech_balloon", "\u{1F4AC}");
    m.insert("speedboat", "\u{1F6A4}");
    m.insert("spider", "\u{1F577}");
    m.insert("spider_web", "\u{1F578}");
    m.insert("spiral_calendar", "\u{1F5D3}");
    m.insert("spiral_notepad", "\u{1F5D2}");
    m.insert("spiral_shell", "\u{1F41A}");
    m.insert("spoon", "\u{1F944}");
    m.insert("sponge", "\u{1F9FD}");
    m.insert("sport_utility_vehicle", "\u{1F699}");
    m.insert("sports_medal", "\u{1F3C5}");
    m.insert("spouting_whale", "\u{1F433}");
    m.insert("squid", "\u{1F991}");
    m.insert("squinting_face_with_tongue", "\u{1F61D}");
    m.insert("stadium", "\u{1F3DF}");
    m.insert("star-struck", "\u{1F929}");
    m.insert("star_and_crescent", "\u{262A}");
    m.insert("star_of_david", "\u{2721}");
    m.insert("station", "\u{1F689}");
    m.insert("steaming_bowl", "\u{1F35C}");
    m.insert("stethoscope", "\u{1FA7A}");
    m.insert("stop_button", "\u{23F9}");
    m.insert("stop_sign", "\u{1F6D1}");
    m.insert("stopwatch", "\u{23F1}");
    m.insert("straight_ruler", "\u{1F4CF}");
    m.insert("strawberry", "\u{1F353}");
    m.insert("studio_microphone", "\u{1F399}");
    m.insert("stuffed_flatbread", "\u{1F959}");
    m.insert("sun", "\u{2600}");
    m.insert("sun_behind_cloud", "\u{26C5}");
    m.insert("sun_behind_large_cloud", "\u{1F325}");
    m.insert("sun_behind_rain_cloud", "\u{1F326}");
    m.insert("sun_behind_small_cloud", "\u{1F324}");
    m.insert("sun_with_face", "\u{1F31E}");
    m.insert("sunflower", "\u{1F33B}");
    m.insert("sunglasses", "\u{1F60E}");
    m.insert("sunrise", "\u{1F305}");
    m.insert("sunrise_over_mountains", "\u{1F304}");
    m.insert("sunset", "\u{1F307}");
    m.insert("superhero", "\u{1F9B8}");
    m.insert("supervillain", "\u{1F9B9}");
    m.insert("sushi", "\u{1F363}");
    m.insert("suspension_railway", "\u{1F69F}");
    m.insert("swan", "\u{1F9A2}");
    m.insert("sweat_droplets", "\u{1F4A6}");
    m.insert("synagogue", "\u{1F54D}");
    m.insert("syringe", "\u{1F489}");
    m.insert("t-shirt", "\u{1F455}");
    m.insert("taco", "\u{1F32E}");
    m.insert("takeout_box", "\u{1F961}");
    m.insert("tanabata_tree", "\u{1F38B}");
    m.insert("tangerine", "\u{1F34A}");
    m.insert("taxi", "\u{1F695}");
    m.insert("teacup_without_handle", "\u{1F375}");
    m.insert("tear-off_calendar", "\u{1F4C6}");
    m.insert("teddy_bear", "\u{1F9F8}");
    m.insert("telephone", "\u{260E}");
    m.insert("telephone_receiver", "\u{1F4DE}");
    m.insert("telescope", "\u{1F52D}");
    m.insert("television", "\u{1F4FA}");
    m.insert("ten-thirty", "\u{1F565}");
    m.insert("ten_o’clock", "\u{1F559}");
    m.insert("tennis", "\u{1F3BE}");
    m.insert("tent", "\u{26FA}");
    m.insert("test_tube", "\u{1F9EA}");
    m.insert("thermometer", "\u{1F321}");
    m.insert("thinking_face", "\u{1F914}");
    m.insert("thought_balloon", "\u{1F4AD}");
    m.insert("thread", "\u{1F9F5}");
    m.insert("three-thirty", "\u{1F55E}");
    m.insert("three_o’clock", "\u{1F552}");
    m.insert("thumbs_down", "\u{1F44E}");
    m.insert("thumbs_down_dark_skin_tone", "\u{1F44E}\u{1F3FF}");
    m.insert("thumbs_down_light_skin_tone", "\u{1F44E}\u{1F3FB}");
    m.insert("thumbs_down_medium-dark_skin_tone", "\u{1F44E}\u{1F3FE}");
    m.insert("thumbs_down_medium-light_skin_tone", "\u{1F44E}\u{1F3FC}");
    m.insert("thumbs_down_medium_skin_tone", "\u{1F44E}\u{1F3FD}");
    m.insert("thumbs_up", "\u{1F44D}");
    m.insert("thumbs_up_dark_skin_tone", "\u{1F44D}\u{1F3FF}");
    m.insert("thumbs_up_light_skin_tone", "\u{1F44D}\u{1F3FB}");
    m.insert("thumbs_up_medium-dark_skin_tone", "\u{1F44D}\u{1F3FE}");
    m.insert("thumbs_up_medium-light_skin_tone", "\u{1F44D}\u{1F3FC}");
    m.insert("thumbs_up_medium_skin_tone", "\u{1F44D}\u{1F3FD}");
    m.insert("ticket", "\u{1F3AB}");
    m.insert("tiger", "\u{1F42F}");
    m.insert("tiger_face", "\u{1F42F}");
    m.insert("timer_clock", "\u{23F2}");
    m.insert("tired_face", "\u{1F62B}");
    m.insert("toolbox", "\u{1F9F0}");
    m.insert("toilet", "\u{1F6BD}");
    m.insert("tomato", "\u{1F345}");
    m.insert("tongue", "\u{1F445}");
    m.insert("tooth", "\u{1F9B7}");
    m.insert("top_hat", "\u{1F3A9}");
    m.insert("tornado", "\u{1F32A}");
    m.insert("trackball", "\u{1F5B2}");
    m.insert("tractor", "\u{1F69C}");
    m.insert("trade_mark", "\u{2122}");
    m.insert("train", "\u{1F68B}");
    m.insert("tram", "\u{1F68A}");
    m.insert("tram_car", "\u{1F68B}");
    m.insert("triangular_flag", "\u{1F6A9}");
    m.insert("triangular_ruler", "\u{1F4D0}");
    m.insert("trident_emblem", "\u{1F531}");
    m.insert("trolleybus", "\u{1F68E}");
    m.insert("trophy", "\u{1F3C6}");
    m.insert("tropical_drink", "\u{1F379}");
    m.insert("tropical_fish", "\u{1F420}");
    m.insert("trumpet", "\u{1F3BA}");
    m.insert("tulip", "\u{1F337}");
    m.insert("tumbler_glass", "\u{1F943}");
    m.insert("turtle", "\u{1F422}");
    m.insert("twelve-thirty", "\u{1F567}");
    m.insert("twelve_o’clock", "\u{1F55B}");
    m.insert("two-hump_camel", "\u{1F42B}");
    m.insert("two-thirty", "\u{1F55D}");
    m.insert("two_hearts", "\u{1F495}");
    m.insert("two_men_holding_hands", "\u{1F46C}");
    m.insert("two_o’clock", "\u{1F551}");
    m.insert("two_women_holding_hands", "\u{1F46D}");
    m.insert("umbrella", "\u{2602}");
    m.insert("umbrella_on_ground", "\u{26F1}");
    m.insert("umbrella_with_rain_drops", "\u{2614}");
    m.insert("unamused_face", "\u{1F612}");
    m.insert("unicorn_face", "\u{1F984}");
    m.insert("unlocked", "\u{1F513}");
    m.insert("up-down_arrow", "\u{2195}");
    m.insert("up-left_arrow", "\u{2196}");
    m.insert("up-right_arrow", "\u{2197}");
    m.insert("up_arrow", "\u{2B06}");
    m.insert("upside-down_face", "\u{1F643}");
    m.insert("upwards_button", "\u{1F53C}");
    m.insert("vampire", "\u{1F9DB}");
    m.insert("vampire_dark_skin_tone", "\u{1F9DB}\u{1F3FF}");
    m.insert("vampire_light_skin_tone", "\u{1F9DB}\u{1F3FB}");
    m.insert("vampire_medium-dark_skin_tone", "\u{1F9DB}\u{1F3FE}");
    m.insert("vampire_medium-light_skin_tone", "\u{1F9DB}\u{1F3FC}");
    m.insert("vampire_medium_skin_tone", "\u{1F9DB}\u{1F3FD}");
    m.insert("vertical_traffic_light", "\u{1F6A6}");
    m.insert("vibration_mode", "\u{1F4F3}");
    m.insert("victory_hand", "\u{270C}");
    m.insert("victory_hand_dark_skin_tone", "\u{270C}\u{1F3FF}");
    m.insert("victory_hand_light_skin_tone", "\u{270C}\u{1F3FB}");
    m.insert("victory_hand_medium-dark_skin_tone", "\u{270C}\u{1F3FE}");
    m.insert("victory_hand_medium-light_skin_tone", "\u{270C}\u{1F3FC}");
    m.insert("victory_hand_medium_skin_tone", "\u{270C}\u{1F3FD}");
    m.insert("video_camera", "\u{1F4F9}");
    m.insert("video_game", "\u{1F3AE}");
    m.insert("videocassette", "\u{1F4FC}");
    m.insert("violin", "\u{1F3BB}");
    m.insert("volcano", "\u{1F30B}");
    m.insert("volleyball", "\u{1F3D0}");
    m.insert("vulcan_salute", "\u{1F596}");
    m.insert("vulcan_salute_dark_skin_tone", "\u{1F596}\u{1F3FF}");
    m.insert("vulcan_salute_light_skin_tone", "\u{1F596}\u{1F3FB}");
    m.insert("vulcan_salute_medium-dark_skin_tone", "\u{1F596}\u{1F3FE}");
    m.insert("vulcan_salute_medium-light_skin_tone", "\u{1F596}\u{1F3FC}");
    m.insert("vulcan_salute_medium_skin_tone", "\u{1F596}\u{1F3FD}");
    m.insert("waffle", "\u{1F9C7}");
    m.insert("waning_crescent_moon", "\u{1F318}");
    m.insert("waning_gibbous_moon", "\u{1F316}");
    m.insert("warning", "\u{26A0}");
    m.insert("wastebasket", "\u{1F5D1}");
    m.insert("watch", "\u{231A}");
    m.insert("water_buffalo", "\u{1F403}");
    m.insert("water_closet", "\u{1F6BE}");
    m.insert("water_wave", "\u{1F30A}");
    m.insert("watermelon", "\u{1F349}");
    m.insert("waving_hand", "\u{1F44B}");
    m.insert("waving_hand_dark_skin_tone", "\u{1F44B}\u{1F3FF}");
    m.insert("waving_hand_light_skin_tone", "\u{1F44B}\u{1F3FB}");
    m.insert("waving_hand_medium-dark_skin_tone", "\u{1F44B}\u{1F3FE}");
    m.insert("waving_hand_medium-light_skin_tone", "\u{1F44B}\u{1F3FC}");
    m.insert("waving_hand_medium_skin_tone", "\u{1F44B}\u{1F3FD}");
    m.insert("wavy_dash", "\u{3030}");
    m.insert("waxing_crescent_moon", "\u{1F312}");
    m.insert("waxing_gibbous_moon", "\u{1F314}");
    m.insert("weary_cat_face", "\u{1F640}");
    m.insert("weary_face", "\u{1F629}");
    m.insert("wedding", "\u{1F492}");
    m.insert("whale", "\u{1F433}");
    m.insert("wheel_of_dharma", "\u{2638}");
    m.insert("wheelchair_symbol", "\u{267F}");
    m.insert("white_circle", "\u{26AA}");
    m.insert("white_exclamation_mark", "\u{2755}");
    m.insert("white_flag", "\u{1F3F3}");
    m.insert("white_flower", "\u{1F4AE}");
    m.insert("white_hair", "\u{1F9B3}");
    m.insert("white-haired_man", "\u{1F468}\u{200D}\u{1F9B3}");
    m.insert("white-haired_woman", "\u{1F469}\u{200D}\u{1F9B3}");
    m.insert("white_heart", "\u{1F90D}");
    m.insert("white_heavy_check_mark", "\u{2705}");
    m.insert("white_large_square", "\u{2B1C}");
    m.insert("white_medium-small_square", "\u{25FD}");
    m.insert("white_medium_square", "\u{25FB}");
    m.insert("white_medium_star", "\u{2B50}");
    m.insert("white_question_mark", "\u{2754}");
    m.insert("white_small_square", "\u{25AB}");
    m.insert("white_square_button", "\u{1F533}");
    m.insert("wilted_flower", "\u{1F940}");
    m.insert("wind_chime", "\u{1F390}");
    m.insert("wind_face", "\u{1F32C}");
    m.insert("wine_glass", "\u{1F377}");
    m.insert("winking_face", "\u{1F609}");
    m.insert("winking_face_with_tongue", "\u{1F61C}");
    m.insert("wolf_face", "\u{1F43A}");
    m.insert("woman", "\u{1F469}");
    m.insert("woman_artist", "\u{1F469}\u{200D}\u{1F3A8}");
    m.insert(
        "woman_artist_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "woman_artist_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "woman_artist_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "woman_artist_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F3A8}",
    );
    m.insert(
        "woman_artist_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F3A8}",
    );
    m.insert("woman_astronaut", "\u{1F469}\u{200D}\u{1F680}");
    m.insert(
        "woman_astronaut_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}",
    );
    m.insert(
        "woman_astronaut_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F680}",
    );
    m.insert(
        "woman_astronaut_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F680}",
    );
    m.insert(
        "woman_astronaut_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F680}",
    );
    m.insert(
        "woman_astronaut_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F680}",
    );
    m.insert("woman_biking", "\u{1F6B4}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_biking_dark_skin_tone",
        "\u{1F6B4}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_biking_light_skin_tone",
        "\u{1F6B4}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_biking_medium-dark_skin_tone",
        "\u{1F6B4}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_biking_medium-light_skin_tone",
        "\u{1F6B4}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_biking_medium_skin_tone",
        "\u{1F6B4}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bouncing_ball",
        "\u{26F9}\u{FE0F}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bouncing_ball_dark_skin_tone",
        "\u{26F9}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bouncing_ball_light_skin_tone",
        "\u{26F9}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bouncing_ball_medium-dark_skin_tone",
        "\u{26F9}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bouncing_ball_medium-light_skin_tone",
        "\u{26F9}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bouncing_ball_medium_skin_tone",
        "\u{26F9}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_bowing", "\u{1F647}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_bowing_dark_skin_tone",
        "\u{1F647}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bowing_light_skin_tone",
        "\u{1F647}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bowing_medium-dark_skin_tone",
        "\u{1F647}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bowing_medium-light_skin_tone",
        "\u{1F647}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_bowing_medium_skin_tone",
        "\u{1F647}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_cartwheeling", "\u{1F938}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_cartwheeling_dark_skin_tone",
        "\u{1F938}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_cartwheeling_light_skin_tone",
        "\u{1F938}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_cartwheeling_medium-dark_skin_tone",
        "\u{1F938}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_cartwheeling_medium-light_skin_tone",
        "\u{1F938}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_cartwheeling_medium_skin_tone",
        "\u{1F938}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_climbing", "\u{1F9D7}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_climbing_dark_skin_tone",
        "\u{1F9D7}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_climbing_light_skin_tone",
        "\u{1F9D7}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_climbing_medium-dark_skin_tone",
        "\u{1F9D7}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_climbing_medium-light_skin_tone",
        "\u{1F9D7}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_climbing_medium_skin_tone",
        "\u{1F9D7}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_construction_worker",
        "\u{1F477}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_construction_worker_dark_skin_tone",
        "\u{1F477}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_construction_worker_light_skin_tone",
        "\u{1F477}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_construction_worker_medium-dark_skin_tone",
        "\u{1F477}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_construction_worker_medium-light_skin_tone",
        "\u{1F477}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_construction_worker_medium_skin_tone",
        "\u{1F477}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_cook", "\u{1F469}\u{200D}\u{1F373}");
    m.insert(
        "woman_cook_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F373}",
    );
    m.insert(
        "woman_cook_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F373}",
    );
    m.insert(
        "woman_cook_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F373}",
    );
    m.insert(
        "woman_cook_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F373}",
    );
    m.insert(
        "woman_cook_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F373}",
    );
    m.insert("woman_dancing", "\u{1F483}");
    m.insert("woman_dancing_dark_skin_tone", "\u{1F483}\u{1F3FF}");
    m.insert("woman_dancing_light_skin_tone", "\u{1F483}\u{1F3FB}");
    m.insert("woman_dancing_medium-dark_skin_tone", "\u{1F483}\u{1F3FE}");
    m.insert("woman_dancing_medium-light_skin_tone", "\u{1F483}\u{1F3FC}");
    m.insert("woman_dancing_medium_skin_tone", "\u{1F483}\u{1F3FD}");
    m.insert("woman_dark_skin_tone", "\u{1F469}\u{1F3FF}");
    m.insert(
        "woman_detective",
        "\u{1F575}\u{FE0F}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_detective_dark_skin_tone",
        "\u{1F575}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_detective_light_skin_tone",
        "\u{1F575}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_detective_medium-dark_skin_tone",
        "\u{1F575}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_detective_medium-light_skin_tone",
        "\u{1F575}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_detective_medium_skin_tone",
        "\u{1F575}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_elf", "\u{1F9DD}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_elf_dark_skin_tone",
        "\u{1F9DD}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_elf_light_skin_tone",
        "\u{1F9DD}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_elf_medium-dark_skin_tone",
        "\u{1F9DD}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_elf_medium-light_skin_tone",
        "\u{1F9DD}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_elf_medium_skin_tone",
        "\u{1F9DD}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_facepalming", "\u{1F926}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_facepalming_dark_skin_tone",
        "\u{1F926}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_facepalming_light_skin_tone",
        "\u{1F926}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_facepalming_medium-dark_skin_tone",
        "\u{1F926}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_facepalming_medium-light_skin_tone",
        "\u{1F926}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_facepalming_medium_skin_tone",
        "\u{1F926}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_factory_worker", "\u{1F469}\u{200D}\u{1F3ED}");
    m.insert(
        "woman_factory_worker_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "woman_factory_worker_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "woman_factory_worker_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "woman_factory_worker_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F3ED}",
    );
    m.insert(
        "woman_factory_worker_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F3ED}",
    );
    m.insert("woman_fairy", "\u{1F9DA}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_fairy_dark_skin_tone",
        "\u{1F9DA}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_fairy_light_skin_tone",
        "\u{1F9DA}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_fairy_medium-dark_skin_tone",
        "\u{1F9DA}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_fairy_medium-light_skin_tone",
        "\u{1F9DA}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_fairy_medium_skin_tone",
        "\u{1F9DA}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_farmer", "\u{1F469}\u{200D}\u{1F33E}");
    m.insert(
        "woman_farmer_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F33E}",
    );
    m.insert(
        "woman_farmer_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F33E}",
    );
    m.insert(
        "woman_farmer_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F33E}",
    );
    m.insert(
        "woman_farmer_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F33E}",
    );
    m.insert(
        "woman_farmer_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F33E}",
    );
    m.insert("woman_firefighter", "\u{1F469}\u{200D}\u{1F692}");
    m.insert(
        "woman_firefighter_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F692}",
    );
    m.insert(
        "woman_firefighter_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F692}",
    );
    m.insert(
        "woman_firefighter_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F692}",
    );
    m.insert(
        "woman_firefighter_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F692}",
    );
    m.insert(
        "woman_firefighter_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F692}",
    );
    m.insert("woman_frowning", "\u{1F64D}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_frowning_dark_skin_tone",
        "\u{1F64D}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_frowning_light_skin_tone",
        "\u{1F64D}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_frowning_medium-dark_skin_tone",
        "\u{1F64D}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_frowning_medium-light_skin_tone",
        "\u{1F64D}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_frowning_medium_skin_tone",
        "\u{1F64D}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_genie", "\u{1F9DE}\u{200D}\u{2640}\u{FE0F}");
    m.insert("woman_gesturing_no", "\u{1F645}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_gesturing_no_dark_skin_tone",
        "\u{1F645}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_no_light_skin_tone",
        "\u{1F645}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_no_medium-dark_skin_tone",
        "\u{1F645}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_no_medium-light_skin_tone",
        "\u{1F645}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_no_medium_skin_tone",
        "\u{1F645}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_gesturing_ok", "\u{1F646}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_gesturing_ok_dark_skin_tone",
        "\u{1F646}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_ok_light_skin_tone",
        "\u{1F646}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_ok_medium-dark_skin_tone",
        "\u{1F646}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_ok_medium-light_skin_tone",
        "\u{1F646}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_gesturing_ok_medium_skin_tone",
        "\u{1F646}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_getting_haircut", "\u{1F487}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_getting_haircut_dark_skin_tone",
        "\u{1F487}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_haircut_light_skin_tone",
        "\u{1F487}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_haircut_medium-dark_skin_tone",
        "\u{1F487}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_haircut_medium-light_skin_tone",
        "\u{1F487}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_haircut_medium_skin_tone",
        "\u{1F487}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_getting_massage", "\u{1F486}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_getting_massage_dark_skin_tone",
        "\u{1F486}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_massage_light_skin_tone",
        "\u{1F486}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_massage_medium-dark_skin_tone",
        "\u{1F486}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_massage_medium-light_skin_tone",
        "\u{1F486}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_getting_massage_medium_skin_tone",
        "\u{1F486}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_golfing", "\u{1F3CC}\u{FE0F}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_golfing_dark_skin_tone",
        "\u{1F3CC}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_golfing_light_skin_tone",
        "\u{1F3CC}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_golfing_medium-dark_skin_tone",
        "\u{1F3CC}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_golfing_medium-light_skin_tone",
        "\u{1F3CC}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_golfing_medium_skin_tone",
        "\u{1F3CC}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_guard", "\u{1F482}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_guard_dark_skin_tone",
        "\u{1F482}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_guard_light_skin_tone",
        "\u{1F482}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_guard_medium-dark_skin_tone",
        "\u{1F482}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_guard_medium-light_skin_tone",
        "\u{1F482}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_guard_medium_skin_tone",
        "\u{1F482}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_health_worker", "\u{1F469}\u{200D}\u{2695}\u{FE0F}");
    m.insert(
        "woman_health_worker_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "woman_health_worker_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "woman_health_worker_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "woman_health_worker_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "woman_health_worker_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{2695}\u{FE0F}",
    );
    m.insert(
        "woman_in_lotus_position",
        "\u{1F9D8}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_lotus_position_dark_skin_tone",
        "\u{1F9D8}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_lotus_position_light_skin_tone",
        "\u{1F9D8}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_lotus_position_medium-dark_skin_tone",
        "\u{1F9D8}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_lotus_position_medium-light_skin_tone",
        "\u{1F9D8}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_lotus_position_medium_skin_tone",
        "\u{1F9D8}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_in_manual_wheelchair", "\u{1F469}\u{200D}\u{1F9BD}");
    m.insert(
        "woman_in_motorized_wheelchair",
        "\u{1F469}\u{200D}\u{1F9BC}",
    );
    m.insert("woman_in_steamy_room", "\u{1F9D6}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_in_steamy_room_dark_skin_tone",
        "\u{1F9D6}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_steamy_room_light_skin_tone",
        "\u{1F9D6}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_steamy_room_medium-dark_skin_tone",
        "\u{1F9D6}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_steamy_room_medium-light_skin_tone",
        "\u{1F9D6}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_in_steamy_room_medium_skin_tone",
        "\u{1F9D6}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_judge", "\u{1F469}\u{200D}\u{2696}\u{FE0F}");
    m.insert(
        "woman_judge_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "woman_judge_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "woman_judge_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "woman_judge_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert(
        "woman_judge_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{2696}\u{FE0F}",
    );
    m.insert("woman_juggling", "\u{1F939}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_juggling_dark_skin_tone",
        "\u{1F939}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_juggling_light_skin_tone",
        "\u{1F939}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_juggling_medium-dark_skin_tone",
        "\u{1F939}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_juggling_medium-light_skin_tone",
        "\u{1F939}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_juggling_medium_skin_tone",
        "\u{1F939}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_lifting_weights",
        "\u{1F3CB}\u{FE0F}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_lifting_weights_dark_skin_tone",
        "\u{1F3CB}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_lifting_weights_light_skin_tone",
        "\u{1F3CB}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_lifting_weights_medium-dark_skin_tone",
        "\u{1F3CB}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_lifting_weights_medium-light_skin_tone",
        "\u{1F3CB}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_lifting_weights_medium_skin_tone",
        "\u{1F3CB}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_light_skin_tone", "\u{1F469}\u{1F3FB}");
    m.insert("woman_mage", "\u{1F9D9}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_mage_dark_skin_tone",
        "\u{1F9D9}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mage_light_skin_tone",
        "\u{1F9D9}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mage_medium-dark_skin_tone",
        "\u{1F9D9}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mage_medium-light_skin_tone",
        "\u{1F9D9}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mage_medium_skin_tone",
        "\u{1F9D9}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_mechanic", "\u{1F469}\u{200D}\u{1F527}");
    m.insert(
        "woman_mechanic_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F527}",
    );
    m.insert(
        "woman_mechanic_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F527}",
    );
    m.insert(
        "woman_mechanic_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F527}",
    );
    m.insert(
        "woman_mechanic_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F527}",
    );
    m.insert(
        "woman_mechanic_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F527}",
    );
    m.insert("woman_medium-dark_skin_tone", "\u{1F469}\u{1F3FE}");
    m.insert("woman_medium-light_skin_tone", "\u{1F469}\u{1F3FC}");
    m.insert("woman_medium_skin_tone", "\u{1F469}\u{1F3FD}");
    m.insert("woman_mountain_biking", "\u{1F6B5}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_mountain_biking_dark_skin_tone",
        "\u{1F6B5}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mountain_biking_light_skin_tone",
        "\u{1F6B5}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mountain_biking_medium-dark_skin_tone",
        "\u{1F6B5}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mountain_biking_medium-light_skin_tone",
        "\u{1F6B5}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_mountain_biking_medium_skin_tone",
        "\u{1F6B5}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_office_worker", "\u{1F469}\u{200D}\u{1F4BC}");
    m.insert(
        "woman_office_worker_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "woman_office_worker_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "woman_office_worker_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "woman_office_worker_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F4BC}",
    );
    m.insert(
        "woman_office_worker_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F4BC}",
    );
    m.insert("woman_pilot", "\u{1F469}\u{200D}\u{2708}\u{FE0F}");
    m.insert(
        "woman_pilot_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "woman_pilot_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "woman_pilot_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "woman_pilot_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "woman_pilot_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{2708}\u{FE0F}",
    );
    m.insert(
        "woman_playing_handball",
        "\u{1F93E}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_handball_dark_skin_tone",
        "\u{1F93E}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_handball_light_skin_tone",
        "\u{1F93E}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_handball_medium-dark_skin_tone",
        "\u{1F93E}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_handball_medium-light_skin_tone",
        "\u{1F93E}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_handball_medium_skin_tone",
        "\u{1F93E}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_water_polo",
        "\u{1F93D}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_water_polo_dark_skin_tone",
        "\u{1F93D}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_water_polo_light_skin_tone",
        "\u{1F93D}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_water_polo_medium-dark_skin_tone",
        "\u{1F93D}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_water_polo_medium-light_skin_tone",
        "\u{1F93D}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_playing_water_polo_medium_skin_tone",
        "\u{1F93D}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_police_officer", "\u{1F46E}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_police_officer_dark_skin_tone",
        "\u{1F46E}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_police_officer_light_skin_tone",
        "\u{1F46E}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_police_officer_medium-dark_skin_tone",
        "\u{1F46E}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_police_officer_medium-light_skin_tone",
        "\u{1F46E}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_police_officer_medium_skin_tone",
        "\u{1F46E}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_pouting", "\u{1F64E}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_pouting_dark_skin_tone",
        "\u{1F64E}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_pouting_light_skin_tone",
        "\u{1F64E}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_pouting_medium-dark_skin_tone",
        "\u{1F64E}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_pouting_medium-light_skin_tone",
        "\u{1F64E}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_pouting_medium_skin_tone",
        "\u{1F64E}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_raising_hand", "\u{1F64B}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_raising_hand_dark_skin_tone",
        "\u{1F64B}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_raising_hand_light_skin_tone",
        "\u{1F64B}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_raising_hand_medium-dark_skin_tone",
        "\u{1F64B}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_raising_hand_medium-light_skin_tone",
        "\u{1F64B}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_raising_hand_medium_skin_tone",
        "\u{1F64B}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_rowing_boat", "\u{1F6A3}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_rowing_boat_dark_skin_tone",
        "\u{1F6A3}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_rowing_boat_light_skin_tone",
        "\u{1F6A3}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_rowing_boat_medium-dark_skin_tone",
        "\u{1F6A3}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_rowing_boat_medium-light_skin_tone",
        "\u{1F6A3}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_rowing_boat_medium_skin_tone",
        "\u{1F6A3}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_running", "\u{1F3C3}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_running_dark_skin_tone",
        "\u{1F3C3}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_running_light_skin_tone",
        "\u{1F3C3}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_running_medium-dark_skin_tone",
        "\u{1F3C3}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_running_medium-light_skin_tone",
        "\u{1F3C3}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_running_medium_skin_tone",
        "\u{1F3C3}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_scientist", "\u{1F469}\u{200D}\u{1F52C}");
    m.insert(
        "woman_scientist_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F52C}",
    );
    m.insert(
        "woman_scientist_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F52C}",
    );
    m.insert(
        "woman_scientist_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F52C}",
    );
    m.insert(
        "woman_scientist_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F52C}",
    );
    m.insert(
        "woman_scientist_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F52C}",
    );
    m.insert("woman_shrugging", "\u{1F937}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_shrugging_dark_skin_tone",
        "\u{1F937}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_shrugging_light_skin_tone",
        "\u{1F937}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_shrugging_medium-dark_skin_tone",
        "\u{1F937}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_shrugging_medium-light_skin_tone",
        "\u{1F937}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_shrugging_medium_skin_tone",
        "\u{1F937}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_singer", "\u{1F469}\u{200D}\u{1F3A4}");
    m.insert(
        "woman_singer_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "woman_singer_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "woman_singer_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "woman_singer_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F3A4}",
    );
    m.insert(
        "woman_singer_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F3A4}",
    );
    m.insert("woman_student", "\u{1F469}\u{200D}\u{1F393}");
    m.insert(
        "woman_student_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F393}",
    );
    m.insert(
        "woman_student_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F393}",
    );
    m.insert(
        "woman_student_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F393}",
    );
    m.insert(
        "woman_student_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F393}",
    );
    m.insert(
        "woman_student_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F393}",
    );
    m.insert("woman_surfing", "\u{1F3C4}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_surfing_dark_skin_tone",
        "\u{1F3C4}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_surfing_light_skin_tone",
        "\u{1F3C4}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_surfing_medium-dark_skin_tone",
        "\u{1F3C4}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_surfing_medium-light_skin_tone",
        "\u{1F3C4}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_surfing_medium_skin_tone",
        "\u{1F3C4}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_swimming", "\u{1F3CA}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_swimming_dark_skin_tone",
        "\u{1F3CA}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_swimming_light_skin_tone",
        "\u{1F3CA}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_swimming_medium-dark_skin_tone",
        "\u{1F3CA}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_swimming_medium-light_skin_tone",
        "\u{1F3CA}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_swimming_medium_skin_tone",
        "\u{1F3CA}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_teacher", "\u{1F469}\u{200D}\u{1F3EB}");
    m.insert(
        "woman_teacher_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "woman_teacher_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "woman_teacher_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "woman_teacher_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F3EB}",
    );
    m.insert(
        "woman_teacher_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F3EB}",
    );
    m.insert("woman_technologist", "\u{1F469}\u{200D}\u{1F4BB}");
    m.insert(
        "woman_technologist_dark_skin_tone",
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "woman_technologist_light_skin_tone",
        "\u{1F469}\u{1F3FB}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "woman_technologist_medium-dark_skin_tone",
        "\u{1F469}\u{1F3FE}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "woman_technologist_medium-light_skin_tone",
        "\u{1F469}\u{1F3FC}\u{200D}\u{1F4BB}",
    );
    m.insert(
        "woman_technologist_medium_skin_tone",
        "\u{1F469}\u{1F3FD}\u{200D}\u{1F4BB}",
    );
    m.insert("woman_tipping_hand", "\u{1F481}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_tipping_hand_dark_skin_tone",
        "\u{1F481}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_tipping_hand_light_skin_tone",
        "\u{1F481}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_tipping_hand_medium-dark_skin_tone",
        "\u{1F481}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_tipping_hand_medium-light_skin_tone",
        "\u{1F481}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_tipping_hand_medium_skin_tone",
        "\u{1F481}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_vampire", "\u{1F9DB}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_vampire_dark_skin_tone",
        "\u{1F9DB}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_vampire_light_skin_tone",
        "\u{1F9DB}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_vampire_medium-dark_skin_tone",
        "\u{1F9DB}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_vampire_medium-light_skin_tone",
        "\u{1F9DB}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_vampire_medium_skin_tone",
        "\u{1F9DB}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_walking", "\u{1F6B6}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_walking_dark_skin_tone",
        "\u{1F6B6}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_walking_light_skin_tone",
        "\u{1F6B6}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_walking_medium-dark_skin_tone",
        "\u{1F6B6}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_walking_medium-light_skin_tone",
        "\u{1F6B6}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_walking_medium_skin_tone",
        "\u{1F6B6}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_wearing_turban", "\u{1F473}\u{200D}\u{2640}\u{FE0F}");
    m.insert(
        "woman_wearing_turban_dark_skin_tone",
        "\u{1F473}\u{1F3FF}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_wearing_turban_light_skin_tone",
        "\u{1F473}\u{1F3FB}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_wearing_turban_medium-dark_skin_tone",
        "\u{1F473}\u{1F3FE}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_wearing_turban_medium-light_skin_tone",
        "\u{1F473}\u{1F3FC}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert(
        "woman_wearing_turban_medium_skin_tone",
        "\u{1F473}\u{1F3FD}\u{200D}\u{2640}\u{FE0F}",
    );
    m.insert("woman_with_headscarf", "\u{1F9D5}");
    m.insert("woman_with_headscarf_dark_skin_tone", "\u{1F9D5}\u{1F3FF}");
    m.insert("woman_with_headscarf_light_skin_tone", "\u{1F9D5}\u{1F3FB}");
    m.insert(
        "woman_with_headscarf_medium-dark_skin_tone",
        "\u{1F9D5}\u{1F3FE}",
    );
    m.insert(
        "woman_with_headscarf_medium-light_skin_tone",
        "\u{1F9D5}\u{1F3FC}",
    );
    m.insert(
        "woman_with_headscarf_medium_skin_tone",
        "\u{1F9D5}\u{1F3FD}",
    );
    m.insert("woman_with_probing_cane", "\u{1F469}\u{200D}\u{1F9AF}");
    m.insert("woman_zombie", "\u{1F9DF}\u{200D}\u{2640}\u{FE0F}");
    m.insert("woman’s_boot", "\u{1F462}");
    m.insert("woman’s_clothes", "\u{1F45A}");
    m.insert("woman’s_hat", "\u{1F452}");
    m.insert("woman’s_sandal", "\u{1F461}");
    m.insert("women_with_bunny_ears", "\u{1F46F}\u{200D}\u{2640}\u{FE0F}");
    m.insert("women_wrestling", "\u{1F93C}\u{200D}\u{2640}\u{FE0F}");
    m.insert("women’s_room", "\u{1F6BA}");
    m.insert("woozy_face", "\u{1F974}");
    m.insert("world_map", "\u{1F5FA}");
    m.insert("worried_face", "\u{1F61F}");
    m.insert("wrapped_gift", "\u{1F381}");
    m.insert("wrench", "\u{1F527}");
    m.insert("writing_hand", "\u{270D}");
    m.insert("writing_hand_dark_skin_tone", "\u{270D}\u{1F3FF}");
    m.insert("writing_hand_light_skin_tone", "\u{270D}\u{1F3FB}");
    m.insert("writing_hand_medium-dark_skin_tone", "\u{270D}\u{1F3FE}");
    m.insert("writing_hand_medium-light_skin_tone", "\u{270D}\u{1F3FC}");
    m.insert("writing_hand_medium_skin_tone", "\u{270D}\u{1F3FD}");
    m.insert("yarn", "\u{1F9F6}");
    m.insert("yawning_face", "\u{1F971}");
    m.insert("yellow_circle", "\u{1F7E1}");
    m.insert("yellow_heart", "\u{1F49B}");
    m.insert("yellow_square", "\u{1F7E8}");
    m.insert("yen_banknote", "\u{1F4B4}");
    m.insert("yo-yo", "\u{1FA80}");
    m.insert("yin_yang", "\u{262F}");
    m.insert("zany_face", "\u{1F92A}");
    m.insert("zebra", "\u{1F993}");
    m.insert("zipper-mouth_face", "\u{1F910}");
    m.insert("zombie", "\u{1F9DF}");
    m.insert("zzz", "\u{1F4A4}");
    m.insert("åland_islands", "\u{1F1E6}\u{1F1FD}");
    m.insert("keycap_asterisk", "*\u{20E3}");
    m.insert("keycap_digit_eight", "8\u{20E3}");
    m.insert("keycap_digit_five", "5\u{20E3}");
    m.insert("keycap_digit_four", "4\u{20E3}");
    m.insert("keycap_digit_nine", "9\u{20E3}");
    m.insert("keycap_digit_one", "1\u{20E3}");
    m.insert("keycap_digit_seven", "7\u{20E3}");
    m.insert("keycap_digit_six", "6\u{20E3}");
    m.insert("keycap_digit_three", "3\u{20E3}");
    m.insert("keycap_digit_two", "2\u{20E3}");
    m.insert("keycap_digit_zero", "0\u{20E3}");
    m.insert("keycap_number_sign", "#\u{20E3}");
    m.insert("light_skin_tone", "\u{1F3FB}");
    m.insert("medium_light_skin_tone", "\u{1F3FC}");
    m.insert("medium_skin_tone", "\u{1F3FD}");
    m.insert("medium_dark_skin_tone", "\u{1F3FE}");
    m.insert("dark_skin_tone", "\u{1F3FF}");
    m.insert("regional_indicator_symbol_letter_a", "\u{1F1E6}");
    m.insert("regional_indicator_symbol_letter_b", "\u{1F1E7}");
    m.insert("regional_indicator_symbol_letter_c", "\u{1F1E8}");
    m.insert("regional_indicator_symbol_letter_d", "\u{1F1E9}");
    m.insert("regional_indicator_symbol_letter_e", "\u{1F1EA}");
    m.insert("regional_indicator_symbol_letter_f", "\u{1F1EB}");
    m.insert("regional_indicator_symbol_letter_g", "\u{1F1EC}");
    m.insert("regional_indicator_symbol_letter_h", "\u{1F1ED}");
    m.insert("regional_indicator_symbol_letter_i", "\u{1F1EE}");
    m.insert("regional_indicator_symbol_letter_j", "\u{1F1EF}");
    m.insert("regional_indicator_symbol_letter_k", "\u{1F1F0}");
    m.insert("regional_indicator_symbol_letter_l", "\u{1F1F1}");
    m.insert("regional_indicator_symbol_letter_m", "\u{1F1F2}");
    m.insert("regional_indicator_symbol_letter_n", "\u{1F1F3}");
    m.insert("regional_indicator_symbol_letter_o", "\u{1F1F4}");
    m.insert("regional_indicator_symbol_letter_p", "\u{1F1F5}");
    m.insert("regional_indicator_symbol_letter_q", "\u{1F1F6}");
    m.insert("regional_indicator_symbol_letter_r", "\u{1F1F7}");
    m.insert("regional_indicator_symbol_letter_s", "\u{1F1F8}");
    m.insert("regional_indicator_symbol_letter_t", "\u{1F1F9}");
    m.insert("regional_indicator_symbol_letter_u", "\u{1F1FA}");
    m.insert("regional_indicator_symbol_letter_v", "\u{1F1FB}");
    m.insert("regional_indicator_symbol_letter_w", "\u{1F1FC}");
    m.insert("regional_indicator_symbol_letter_x", "\u{1F1FD}");
    m.insert("regional_indicator_symbol_letter_y", "\u{1F1FE}");
    m.insert("regional_indicator_symbol_letter_z", "\u{1F1FF}");
    m.insert("airplane_arriving", "\u{1F6EC}");
    m.insert("space_invader", "\u{1F47E}");
    m.insert("football", "\u{1F3C8}");
    m.insert("anger", "\u{1F4A2}");
    m.insert("angry", "\u{1F620}");
    m.insert("anguished", "\u{1F627}");
    m.insert("signal_strength", "\u{1F4F6}");
    m.insert("arrows_counterclockwise", "\u{1F504}");
    m.insert("arrow_heading_down", "\u{2935}");
    m.insert("arrow_heading_up", "\u{2934}");
    m.insert("art", "\u{1F3A8}");
    m.insert("astonished", "\u{1F632}");
    m.insert("athletic_shoe", "\u{1F45F}");
    m.insert("atm", "\u{1F3E7}");
    m.insert("car", "\u{1F697}");
    m.insert("red_car", "\u{1F697}");
    m.insert("angel", "\u{1F47C}");
    m.insert("back", "\u{1F519}");
    m.insert("badminton_racquet_and_shuttlecock", "\u{1F3F8}");
    m.insert("dollar", "\u{1F4B5}");
    m.insert("euro", "\u{1F4B6}");
    m.insert("pound", "\u{1F4B7}");
    m.insert("yen", "\u{1F4B4}");
    m.insert("barber", "\u{1F488}");
    m.insert("bath", "\u{1F6C0}");
    m.insert("bear", "\u{1F43B}");
    m.insert("heartbeat", "\u{1F493}");
    m.insert("beer", "\u{1F37A}");
    m.insert("no_bell", "\u{1F515}");
    m.insert("bento", "\u{1F371}");
    m.insert("bike", "\u{1F6B2}");
    m.insert("bicyclist", "\u{1F6B4}");
    m.insert("8ball", "\u{1F3B1}");
    m.insert("biohazard_sign", "\u{2623}");
    m.insert("birthday", "\u{1F382}");
    m.insert("black_circle_for_record", "\u{23FA}");
    m.insert("clubs", "\u{2663}");
    m.insert("diamonds", "\u{2666}");
    m.insert("arrow_double_down", "\u{23EC}");
    m.insert("hearts", "\u{2665}");
    m.insert("rewind", "\u{23EA}");
    m.insert(
        "black_left__pointing_double_triangle_with_vertical_bar",
        "\u{23EE}",
    );
    m.insert("arrow_backward", "\u{25C0}");
    m.insert("black_medium_small_square", "\u{25FE}");
    m.insert("question", "\u{2753}");
    m.insert("fast_forward", "\u{23E9}");
    m.insert(
        "black_right__pointing_double_triangle_with_vertical_bar",
        "\u{23ED}",
    );
    m.insert("arrow_forward", "\u{25B6}");
    m.insert(
        "black_right__pointing_triangle_with_double_vertical_bar",
        "\u{23EF}",
    );
    m.insert("arrow_right", "\u{27A1}");
    m.insert("spades", "\u{2660}");
    m.insert("black_square_for_stop", "\u{23F9}");
    m.insert("sunny", "\u{2600}");
    m.insert("phone", "\u{260E}");
    m.insert("recycle", "\u{267B}");
    m.insert("arrow_double_up", "\u{23EB}");
    m.insert("busstop", "\u{1F68F}");
    m.insert("date", "\u{1F4C5}");
    m.insert("flags", "\u{1F38F}");
    m.insert("cat2", "\u{1F408}");
    m.insert("joy_cat", "\u{1F639}");
    m.insert("smirk_cat", "\u{1F63C}");
    m.insert("chart_with_downwards_trend", "\u{1F4C9}");
    m.insert("chart_with_upwards_trend", "\u{1F4C8}");
    m.insert("chart", "\u{1F4B9}");
    m.insert("mega", "\u{1F4E3}");
    m.insert("checkered_flag", "\u{1F3C1}");
    m.insert("accept", "\u{1F251}");
    m.insert("ideograph_advantage", "\u{1F250}");
    m.insert("congratulations", "\u{3297}");
    m.insert("secret", "\u{3299}");
    m.insert("m", "\u{24C2}");
    m.insert("city_sunset", "\u{1F306}");
    m.insert("clapper", "\u{1F3AC}");
    m.insert("clap", "\u{1F44F}");
    m.insert("beers", "\u{1F37B}");
    m.insert("clock830", "\u{1F563}");
    m.insert("clock8", "\u{1F557}");
    m.insert("clock1130", "\u{1F566}");
    m.insert("clock11", "\u{1F55A}");
    m.insert("clock530", "\u{1F560}");
    m.insert("clock5", "\u{1F554}");
    m.insert("clock430", "\u{1F55F}");
    m.insert("clock4", "\u{1F553}");
    m.insert("clock930", "\u{1F564}");
    m.insert("clock9", "\u{1F558}");
    m.insert("clock130", "\u{1F55C}");
    m.insert("clock1", "\u{1F550}");
    m.insert("clock730", "\u{1F562}");
    m.insert("clock7", "\u{1F556}");
    m.insert("clock630", "\u{1F561}");
    m.insert("clock6", "\u{1F555}");
    m.insert("clock1030", "\u{1F565}");
    m.insert("clock10", "\u{1F559}");
    m.insert("clock330", "\u{1F55E}");
    m.insert("clock3", "\u{1F552}");
    m.insert("clock1230", "\u{1F567}");
    m.insert("clock12", "\u{1F55B}");
    m.insert("clock230", "\u{1F55D}");
    m.insert("clock2", "\u{1F551}");
    m.insert("arrows_clockwise", "\u{1F503}");
    m.insert("repeat", "\u{1F501}");
    m.insert("repeat_one", "\u{1F502}");
    m.insert("closed_lock_with_key", "\u{1F510}");
    m.insert("mailbox_closed", "\u{1F4EA}");
    m.insert("mailbox", "\u{1F4EB}");
    m.insert("cloud_with_tornado", "\u{1F32A}");
    m.insert("cocktail", "\u{1F378}");
    m.insert("boom", "\u{1F4A5}");
    m.insert("compression", "\u{1F5DC}");
    m.insert("confounded", "\u{1F616}");
    m.insert("confused", "\u{1F615}");
    m.insert("rice", "\u{1F35A}");
    m.insert("cow2", "\u{1F404}");
    m.insert("cricket_bat_and_ball", "\u{1F3CF}");
    m.insert("x", "\u{274C}");
    m.insert("cry", "\u{1F622}");
    m.insert("curry", "\u{1F35B}");
    m.insert("dagger_knife", "\u{1F5E1}");
    m.insert("dancer", "\u{1F483}");
    m.insert("dark_sunglasses", "\u{1F576}");
    m.insert("dash", "\u{1F4A8}");
    m.insert("truck", "\u{1F69A}");
    m.insert("derelict_house_building", "\u{1F3DA}");
    m.insert("diamond_shape_with_a_dot_inside", "\u{1F4A0}");
    m.insert("dart", "\u{1F3AF}");
    m.insert("disappointed_relieved", "\u{1F625}");
    m.insert("disappointed", "\u{1F61E}");
    m.insert("do_not_litter", "\u{1F6AF}");
    m.insert("dog2", "\u{1F415}");
    m.insert("flipper", "\u{1F42C}");
    m.insert("loop", "\u{27BF}");
    m.insert("bangbang", "\u{203C}");
    m.insert("double_vertical_bar", "\u{23F8}");
    m.insert("dove_of_peace", "\u{1F54A}");
    m.insert("small_red_triangle_down", "\u{1F53B}");
    m.insert("arrow_down_small", "\u{1F53D}");
    m.insert("arrow_down", "\u{2B07}");
    m.insert("dromedary_camel", "\u{1F42A}");
    m.insert("e__mail", "\u{1F4E7}");
    m.insert("corn", "\u{1F33D}");
    m.insert("ear_of_rice", "\u{1F33E}");
    m.insert("earth_americas", "\u{1F30E}");
    m.insert("earth_asia", "\u{1F30F}");
    m.insert("earth_africa", "\u{1F30D}");
    m.insert("eight_pointed_black_star", "\u{2734}");
    m.insert("eight_spoked_asterisk", "\u{2733}");
    m.insert("eject_symbol", "\u{23CF}");
    m.insert("bulb", "\u{1F4A1}");
    m.insert("emoji_modifier_fitzpatrick_type__1__2", "\u{1F3FB}");
    m.insert("emoji_modifier_fitzpatrick_type__3", "\u{1F3FC}");
    m.insert("emoji_modifier_fitzpatrick_type__4", "\u{1F3FD}");
    m.insert("emoji_modifier_fitzpatrick_type__5", "\u{1F3FE}");
    m.insert("emoji_modifier_fitzpatrick_type__6", "\u{1F3FF}");
    m.insert("end", "\u{1F51A}");
    m.insert("email", "\u{2709}");
    m.insert("european_castle", "\u{1F3F0}");
    m.insert("european_post_office", "\u{1F3E4}");
    m.insert("interrobang", "\u{2049}");
    m.insert("expressionless", "\u{1F611}");
    m.insert("eyeglasses", "\u{1F453}");
    m.insert("massage", "\u{1F486}");
    m.insert("yum", "\u{1F60B}");
    m.insert("scream", "\u{1F631}");
    m.insert("kissing_heart", "\u{1F618}");
    m.insert("sweat", "\u{1F613}");
    m.insert("face_with_head__bandage", "\u{1F915}");
    m.insert("triumph", "\u{1F624}");
    m.insert("mask", "\u{1F637}");
    m.insert("no_good", "\u{1F645}");
    m.insert("ok_woman", "\u{1F646}");
    m.insert("open_mouth", "\u{1F62E}");
    m.insert("cold_sweat", "\u{1F630}");
    m.insert("stuck_out_tongue", "\u{1F61B}");
    m.insert("stuck_out_tongue_closed_eyes", "\u{1F61D}");
    m.insert("stuck_out_tongue_winking_eye", "\u{1F61C}");
    m.insert("joy", "\u{1F602}");
    m.insert("no_mouth", "\u{1F636}");
    m.insert("santa", "\u{1F385}");
    m.insert("fax", "\u{1F4E0}");
    m.insert("fearful", "\u{1F628}");
    m.insert("field_hockey_stick_and_ball", "\u{1F3D1}");
    m.insert("first_quarter_moon_with_face", "\u{1F31B}");
    m.insert("fish_cake", "\u{1F365}");
    m.insert("fishing_pole_and_fish", "\u{1F3A3}");
    m.insert("facepunch", "\u{1F44A}");
    m.insert("punch", "\u{1F44A}");
    m.insert("flag_for_afghanistan", "\u{1F1E6}\u{1F1EB}");
    m.insert("flag_for_albania", "\u{1F1E6}\u{1F1F1}");
    m.insert("flag_for_algeria", "\u{1F1E9}\u{1F1FF}");
    m.insert("flag_for_american_samoa", "\u{1F1E6}\u{1F1F8}");
    m.insert("flag_for_andorra", "\u{1F1E6}\u{1F1E9}");
    m.insert("flag_for_angola", "\u{1F1E6}\u{1F1F4}");
    m.insert("flag_for_anguilla", "\u{1F1E6}\u{1F1EE}");
    m.insert("flag_for_antarctica", "\u{1F1E6}\u{1F1F6}");
    m.insert("flag_for_antigua_&_barbuda", "\u{1F1E6}\u{1F1EC}");
    m.insert("flag_for_argentina", "\u{1F1E6}\u{1F1F7}");
    m.insert("flag_for_armenia", "\u{1F1E6}\u{1F1F2}");
    m.insert("flag_for_aruba", "\u{1F1E6}\u{1F1FC}");
    m.insert("flag_for_ascension_island", "\u{1F1E6}\u{1F1E8}");
    m.insert("flag_for_australia", "\u{1F1E6}\u{1F1FA}");
    m.insert("flag_for_austria", "\u{1F1E6}\u{1F1F9}");
    m.insert("flag_for_azerbaijan", "\u{1F1E6}\u{1F1FF}");
    m.insert("flag_for_bahamas", "\u{1F1E7}\u{1F1F8}");
    m.insert("flag_for_bahrain", "\u{1F1E7}\u{1F1ED}");
    m.insert("flag_for_bangladesh", "\u{1F1E7}\u{1F1E9}");
    m.insert("flag_for_barbados", "\u{1F1E7}\u{1F1E7}");
    m.insert("flag_for_belarus", "\u{1F1E7}\u{1F1FE}");
    m.insert("flag_for_belgium", "\u{1F1E7}\u{1F1EA}");
    m.insert("flag_for_belize", "\u{1F1E7}\u{1F1FF}");
    m.insert("flag_for_benin", "\u{1F1E7}\u{1F1EF}");
    m.insert("flag_for_bermuda", "\u{1F1E7}\u{1F1F2}");
    m.insert("flag_for_bhutan", "\u{1F1E7}\u{1F1F9}");
    m.insert("flag_for_bolivia", "\u{1F1E7}\u{1F1F4}");
    m.insert("flag_for_bosnia_&_herzegovina", "\u{1F1E7}\u{1F1E6}");
    m.insert("flag_for_botswana", "\u{1F1E7}\u{1F1FC}");
    m.insert("flag_for_bouvet_island", "\u{1F1E7}\u{1F1FB}");
    m.insert("flag_for_brazil", "\u{1F1E7}\u{1F1F7}");
    m.insert(
        "flag_for_british_indian_ocean_territory",
        "\u{1F1EE}\u{1F1F4}",
    );
    m.insert("flag_for_british_virgin_islands", "\u{1F1FB}\u{1F1EC}");
    m.insert("flag_for_brunei", "\u{1F1E7}\u{1F1F3}");
    m.insert("flag_for_bulgaria", "\u{1F1E7}\u{1F1EC}");
    m.insert("flag_for_burkina_faso", "\u{1F1E7}\u{1F1EB}");
    m.insert("flag_for_burundi", "\u{1F1E7}\u{1F1EE}");
    m.insert("flag_for_cambodia", "\u{1F1F0}\u{1F1ED}");
    m.insert("flag_for_cameroon", "\u{1F1E8}\u{1F1F2}");
    m.insert("flag_for_canada", "\u{1F1E8}\u{1F1E6}");
    m.insert("flag_for_canary_islands", "\u{1F1EE}\u{1F1E8}");
    m.insert("flag_for_cape_verde", "\u{1F1E8}\u{1F1FB}");
    m.insert("flag_for_caribbean_netherlands", "\u{1F1E7}\u{1F1F6}");
    m.insert("flag_for_cayman_islands", "\u{1F1F0}\u{1F1FE}");
    m.insert("flag_for_central_african_republic", "\u{1F1E8}\u{1F1EB}");
    m.insert("flag_for_ceuta_&_melilla", "\u{1F1EA}\u{1F1E6}");
    m.insert("flag_for_chad", "\u{1F1F9}\u{1F1E9}");
    m.insert("flag_for_chile", "\u{1F1E8}\u{1F1F1}");
    m.insert("flag_for_china", "\u{1F1E8}\u{1F1F3}");
    m.insert("flag_for_christmas_island", "\u{1F1E8}\u{1F1FD}");
    m.insert("flag_for_clipperton_island", "\u{1F1E8}\u{1F1F5}");
    m.insert("flag_for_cocos__islands", "\u{1F1E8}\u{1F1E8}");
    m.insert("flag_for_colombia", "\u{1F1E8}\u{1F1F4}");
    m.insert("flag_for_comoros", "\u{1F1F0}\u{1F1F2}");
    m.insert("flag_for_congo____brazzaville", "\u{1F1E8}\u{1F1EC}");
    m.insert("flag_for_congo____kinshasa", "\u{1F1E8}\u{1F1E9}");
    m.insert("flag_for_cook_islands", "\u{1F1E8}\u{1F1F0}");
    m.insert("flag_for_costa_rica", "\u{1F1E8}\u{1F1F7}");
    m.insert("flag_for_croatia", "\u{1F1ED}\u{1F1F7}");
    m.insert("flag_for_cuba", "\u{1F1E8}\u{1F1FA}");
    m.insert("flag_for_curaçao", "\u{1F1E8}\u{1F1FC}");
    m.insert("flag_for_cyprus", "\u{1F1E8}\u{1F1FE}");
    m.insert("flag_for_czech_republic", "\u{1F1E8}\u{1F1FF}");
    m.insert("flag_for_côte_d’ivoire", "\u{1F1E8}\u{1F1EE}");
    m.insert("flag_for_denmark", "\u{1F1E9}\u{1F1F0}");
    m.insert("flag_for_diego_garcia", "\u{1F1E9}\u{1F1EC}");
    m.insert("flag_for_djibouti", "\u{1F1E9}\u{1F1EF}");
    m.insert("flag_for_dominica", "\u{1F1E9}\u{1F1F2}");
    m.insert("flag_for_dominican_republic", "\u{1F1E9}\u{1F1F4}");
    m.insert("flag_for_ecuador", "\u{1F1EA}\u{1F1E8}");
    m.insert("flag_for_egypt", "\u{1F1EA}\u{1F1EC}");
    m.insert("flag_for_el_salvador", "\u{1F1F8}\u{1F1FB}");
    m.insert("flag_for_equatorial_guinea", "\u{1F1EC}\u{1F1F6}");
    m.insert("flag_for_eritrea", "\u{1F1EA}\u{1F1F7}");
    m.insert("flag_for_estonia", "\u{1F1EA}\u{1F1EA}");
    m.insert("flag_for_ethiopia", "\u{1F1EA}\u{1F1F9}");
    m.insert("flag_for_european_union", "\u{1F1EA}\u{1F1FA}");
    m.insert("flag_for_falkland_islands", "\u{1F1EB}\u{1F1F0}");
    m.insert("flag_for_faroe_islands", "\u{1F1EB}\u{1F1F4}");
    m.insert("flag_for_fiji", "\u{1F1EB}\u{1F1EF}");
    m.insert("flag_for_finland", "\u{1F1EB}\u{1F1EE}");
    m.insert("flag_for_france", "\u{1F1EB}\u{1F1F7}");
    m.insert("flag_for_french_guiana", "\u{1F1EC}\u{1F1EB}");
    m.insert("flag_for_french_polynesia", "\u{1F1F5}\u{1F1EB}");
    m.insert("flag_for_french_southern_territories", "\u{1F1F9}\u{1F1EB}");
    m.insert("flag_for_gabon", "\u{1F1EC}\u{1F1E6}");
    m.insert("flag_for_gambia", "\u{1F1EC}\u{1F1F2}");
    m.insert("flag_for_georgia", "\u{1F1EC}\u{1F1EA}");
    m.insert("flag_for_germany", "\u{1F1E9}\u{1F1EA}");
    m.insert("flag_for_ghana", "\u{1F1EC}\u{1F1ED}");
    m.insert("flag_for_gibraltar", "\u{1F1EC}\u{1F1EE}");
    m.insert("flag_for_greece", "\u{1F1EC}\u{1F1F7}");
    m.insert("flag_for_greenland", "\u{1F1EC}\u{1F1F1}");
    m.insert("flag_for_grenada", "\u{1F1EC}\u{1F1E9}");
    m.insert("flag_for_guadeloupe", "\u{1F1EC}\u{1F1F5}");
    m.insert("flag_for_guam", "\u{1F1EC}\u{1F1FA}");
    m.insert("flag_for_guatemala", "\u{1F1EC}\u{1F1F9}");
    m.insert("flag_for_guernsey", "\u{1F1EC}\u{1F1EC}");
    m.insert("flag_for_guinea", "\u{1F1EC}\u{1F1F3}");
    m.insert("flag_for_guinea__bissau", "\u{1F1EC}\u{1F1FC}");
    m.insert("flag_for_guyana", "\u{1F1EC}\u{1F1FE}");
    m.insert("flag_for_haiti", "\u{1F1ED}\u{1F1F9}");
    m.insert("flag_for_heard_&_mcdonald_islands", "\u{1F1ED}\u{1F1F2}");
    m.insert("flag_for_honduras", "\u{1F1ED}\u{1F1F3}");
    m.insert("flag_for_hong_kong", "\u{1F1ED}\u{1F1F0}");
    m.insert("flag_for_hungary", "\u{1F1ED}\u{1F1FA}");
    m.insert("flag_for_iceland", "\u{1F1EE}\u{1F1F8}");
    m.insert("flag_for_india", "\u{1F1EE}\u{1F1F3}");
    m.insert("flag_for_indonesia", "\u{1F1EE}\u{1F1E9}");
    m.insert("flag_for_iran", "\u{1F1EE}\u{1F1F7}");
    m.insert("flag_for_iraq", "\u{1F1EE}\u{1F1F6}");
    m.insert("flag_for_ireland", "\u{1F1EE}\u{1F1EA}");
    m.insert("flag_for_isle_of_man", "\u{1F1EE}\u{1F1F2}");
    m.insert("flag_for_israel", "\u{1F1EE}\u{1F1F1}");
    m.insert("flag_for_italy", "\u{1F1EE}\u{1F1F9}");
    m.insert("flag_for_jamaica", "\u{1F1EF}\u{1F1F2}");
    m.insert("flag_for_japan", "\u{1F1EF}\u{1F1F5}");
    m.insert("flag_for_jersey", "\u{1F1EF}\u{1F1EA}");
    m.insert("flag_for_jordan", "\u{1F1EF}\u{1F1F4}");
    m.insert("flag_for_kazakhstan", "\u{1F1F0}\u{1F1FF}");
    m.insert("flag_for_kenya", "\u{1F1F0}\u{1F1EA}");
    m.insert("flag_for_kiribati", "\u{1F1F0}\u{1F1EE}");
    m.insert("flag_for_kosovo", "\u{1F1FD}\u{1F1F0}");
    m.insert("flag_for_kuwait", "\u{1F1F0}\u{1F1FC}");
    m.insert("flag_for_kyrgyzstan", "\u{1F1F0}\u{1F1EC}");
    m.insert("flag_for_laos", "\u{1F1F1}\u{1F1E6}");
    m.insert("flag_for_latvia", "\u{1F1F1}\u{1F1FB}");
    m.insert("flag_for_lebanon", "\u{1F1F1}\u{1F1E7}");
    m.insert("flag_for_lesotho", "\u{1F1F1}\u{1F1F8}");
    m.insert("flag_for_liberia", "\u{1F1F1}\u{1F1F7}");
    m.insert("flag_for_libya", "\u{1F1F1}\u{1F1FE}");
    m.insert("flag_for_liechtenstein", "\u{1F1F1}\u{1F1EE}");
    m.insert("flag_for_lithuania", "\u{1F1F1}\u{1F1F9}");
    m.insert("flag_for_luxembourg", "\u{1F1F1}\u{1F1FA}");
    m.insert("flag_for_macau", "\u{1F1F2}\u{1F1F4}");
    m.insert("flag_for_macedonia", "\u{1F1F2}\u{1F1F0}");
    m.insert("flag_for_madagascar", "\u{1F1F2}\u{1F1EC}");
    m.insert("flag_for_malawi", "\u{1F1F2}\u{1F1FC}");
    m.insert("flag_for_malaysia", "\u{1F1F2}\u{1F1FE}");
    m.insert("flag_for_maldives", "\u{1F1F2}\u{1F1FB}");
    m.insert("flag_for_mali", "\u{1F1F2}\u{1F1F1}");
    m.insert("flag_for_malta", "\u{1F1F2}\u{1F1F9}");
    m.insert("flag_for_marshall_islands", "\u{1F1F2}\u{1F1ED}");
    m.insert("flag_for_martinique", "\u{1F1F2}\u{1F1F6}");
    m.insert("flag_for_mauritania", "\u{1F1F2}\u{1F1F7}");
    m.insert("flag_for_mauritius", "\u{1F1F2}\u{1F1FA}");
    m.insert("flag_for_mayotte", "\u{1F1FE}\u{1F1F9}");
    m.insert("flag_for_mexico", "\u{1F1F2}\u{1F1FD}");
    m.insert("flag_for_micronesia", "\u{1F1EB}\u{1F1F2}");
    m.insert("flag_for_moldova", "\u{1F1F2}\u{1F1E9}");
    m.insert("flag_for_monaco", "\u{1F1F2}\u{1F1E8}");
    m.insert("flag_for_mongolia", "\u{1F1F2}\u{1F1F3}");
    m.insert("flag_for_montenegro", "\u{1F1F2}\u{1F1EA}");
    m.insert("flag_for_montserrat", "\u{1F1F2}\u{1F1F8}");
    m.insert("flag_for_morocco", "\u{1F1F2}\u{1F1E6}");
    m.insert("flag_for_mozambique", "\u{1F1F2}\u{1F1FF}");
    m.insert("flag_for_myanmar", "\u{1F1F2}\u{1F1F2}");
    m.insert("flag_for_namibia", "\u{1F1F3}\u{1F1E6}");
    m.insert("flag_for_nauru", "\u{1F1F3}\u{1F1F7}");
    m.insert("flag_for_nepal", "\u{1F1F3}\u{1F1F5}");
    m.insert("flag_for_netherlands", "\u{1F1F3}\u{1F1F1}");
    m.insert("flag_for_new_caledonia", "\u{1F1F3}\u{1F1E8}");
    m.insert("flag_for_new_zealand", "\u{1F1F3}\u{1F1FF}");
    m.insert("flag_for_nicaragua", "\u{1F1F3}\u{1F1EE}");
    m.insert("flag_for_niger", "\u{1F1F3}\u{1F1EA}");
    m.insert("flag_for_nigeria", "\u{1F1F3}\u{1F1EC}");
    m.insert("flag_for_niue", "\u{1F1F3}\u{1F1FA}");
    m.insert("flag_for_norfolk_island", "\u{1F1F3}\u{1F1EB}");
    m.insert("flag_for_north_korea", "\u{1F1F0}\u{1F1F5}");
    m.insert("flag_for_northern_mariana_islands", "\u{1F1F2}\u{1F1F5}");
    m.insert("flag_for_norway", "\u{1F1F3}\u{1F1F4}");
    m.insert("flag_for_oman", "\u{1F1F4}\u{1F1F2}");
    m.insert("flag_for_pakistan", "\u{1F1F5}\u{1F1F0}");
    m.insert("flag_for_palau", "\u{1F1F5}\u{1F1FC}");
    m.insert("flag_for_palestinian_territories", "\u{1F1F5}\u{1F1F8}");
    m.insert("flag_for_panama", "\u{1F1F5}\u{1F1E6}");
    m.insert("flag_for_papua_new_guinea", "\u{1F1F5}\u{1F1EC}");
    m.insert("flag_for_paraguay", "\u{1F1F5}\u{1F1FE}");
    m.insert("flag_for_peru", "\u{1F1F5}\u{1F1EA}");
    m.insert("flag_for_philippines", "\u{1F1F5}\u{1F1ED}");
    m.insert("flag_for_pitcairn_islands", "\u{1F1F5}\u{1F1F3}");
    m.insert("flag_for_poland", "\u{1F1F5}\u{1F1F1}");
    m.insert("flag_for_portugal", "\u{1F1F5}\u{1F1F9}");
    m.insert("flag_for_puerto_rico", "\u{1F1F5}\u{1F1F7}");
    m.insert("flag_for_qatar", "\u{1F1F6}\u{1F1E6}");
    m.insert("flag_for_romania", "\u{1F1F7}\u{1F1F4}");
    m.insert("flag_for_russia", "\u{1F1F7}\u{1F1FA}");
    m.insert("flag_for_rwanda", "\u{1F1F7}\u{1F1FC}");
    m.insert("flag_for_réunion", "\u{1F1F7}\u{1F1EA}");
    m.insert("flag_for_samoa", "\u{1F1FC}\u{1F1F8}");
    m.insert("flag_for_san_marino", "\u{1F1F8}\u{1F1F2}");
    m.insert("flag_for_saudi_arabia", "\u{1F1F8}\u{1F1E6}");
    m.insert("flag_for_senegal", "\u{1F1F8}\u{1F1F3}");
    m.insert("flag_for_serbia", "\u{1F1F7}\u{1F1F8}");
    m.insert("flag_for_seychelles", "\u{1F1F8}\u{1F1E8}");
    m.insert("flag_for_sierra_leone", "\u{1F1F8}\u{1F1F1}");
    m.insert("flag_for_singapore", "\u{1F1F8}\u{1F1EC}");
    m.insert("flag_for_sint_maarten", "\u{1F1F8}\u{1F1FD}");
    m.insert("flag_for_slovakia", "\u{1F1F8}\u{1F1F0}");
    m.insert("flag_for_slovenia", "\u{1F1F8}\u{1F1EE}");
    m.insert("flag_for_solomon_islands", "\u{1F1F8}\u{1F1E7}");
    m.insert("flag_for_somalia", "\u{1F1F8}\u{1F1F4}");
    m.insert("flag_for_south_africa", "\u{1F1FF}\u{1F1E6}");
    m.insert(
        "flag_for_south_georgia_&_south_sandwich_islands",
        "\u{1F1EC}\u{1F1F8}",
    );
    m.insert("flag_for_south_korea", "\u{1F1F0}\u{1F1F7}");
    m.insert("flag_for_south_sudan", "\u{1F1F8}\u{1F1F8}");
    m.insert("flag_for_spain", "\u{1F1EA}\u{1F1F8}");
    m.insert("flag_for_sri_lanka", "\u{1F1F1}\u{1F1F0}");
    m.insert("flag_for_st._barthélemy", "\u{1F1E7}\u{1F1F1}");
    m.insert("flag_for_st._helena", "\u{1F1F8}\u{1F1ED}");
    m.insert("flag_for_st._kitts_&_nevis", "\u{1F1F0}\u{1F1F3}");
    m.insert("flag_for_st._lucia", "\u{1F1F1}\u{1F1E8}");
    m.insert("flag_for_st._martin", "\u{1F1F2}\u{1F1EB}");
    m.insert("flag_for_st._pierre_&_miquelon", "\u{1F1F5}\u{1F1F2}");
    m.insert("flag_for_st._vincent_&_grenadines", "\u{1F1FB}\u{1F1E8}");
    m.insert("flag_for_sudan", "\u{1F1F8}\u{1F1E9}");
    m.insert("flag_for_suriname", "\u{1F1F8}\u{1F1F7}");
    m.insert("flag_for_svalbard_&_jan_mayen", "\u{1F1F8}\u{1F1EF}");
    m.insert("flag_for_swaziland", "\u{1F1F8}\u{1F1FF}");
    m.insert("flag_for_sweden", "\u{1F1F8}\u{1F1EA}");
    m.insert("flag_for_switzerland", "\u{1F1E8}\u{1F1ED}");
    m.insert("flag_for_syria", "\u{1F1F8}\u{1F1FE}");
    m.insert("flag_for_são_tomé_&_príncipe", "\u{1F1F8}\u{1F1F9}");
    m.insert("flag_for_taiwan", "\u{1F1F9}\u{1F1FC}");
    m.insert("flag_for_tajikistan", "\u{1F1F9}\u{1F1EF}");
    m.insert("flag_for_tanzania", "\u{1F1F9}\u{1F1FF}");
    m.insert("flag_for_thailand", "\u{1F1F9}\u{1F1ED}");
    m.insert("flag_for_timor__leste", "\u{1F1F9}\u{1F1F1}");
    m.insert("flag_for_togo", "\u{1F1F9}\u{1F1EC}");
    m.insert("flag_for_tokelau", "\u{1F1F9}\u{1F1F0}");
    m.insert("flag_for_tonga", "\u{1F1F9}\u{1F1F4}");
    m.insert("flag_for_trinidad_&_tobago", "\u{1F1F9}\u{1F1F9}");
    m.insert("flag_for_tristan_da_cunha", "\u{1F1F9}\u{1F1E6}");
    m.insert("flag_for_tunisia", "\u{1F1F9}\u{1F1F3}");
    m.insert("flag_for_turkey", "\u{1F1F9}\u{1F1F7}");
    m.insert("flag_for_turkmenistan", "\u{1F1F9}\u{1F1F2}");
    m.insert("flag_for_turks_&_caicos_islands", "\u{1F1F9}\u{1F1E8}");
    m.insert("flag_for_tuvalu", "\u{1F1F9}\u{1F1FB}");
    m.insert("flag_for_u.s._outlying_islands", "\u{1F1FA}\u{1F1F2}");
    m.insert("flag_for_u.s._virgin_islands", "\u{1F1FB}\u{1F1EE}");
    m.insert("flag_for_uganda", "\u{1F1FA}\u{1F1EC}");
    m.insert("flag_for_ukraine", "\u{1F1FA}\u{1F1E6}");
    m.insert("flag_for_united_arab_emirates", "\u{1F1E6}\u{1F1EA}");
    m.insert("flag_for_united_kingdom", "\u{1F1EC}\u{1F1E7}");
    m.insert("flag_for_united_states", "\u{1F1FA}\u{1F1F8}");
    m.insert("flag_for_uruguay", "\u{1F1FA}\u{1F1FE}");
    m.insert("flag_for_uzbekistan", "\u{1F1FA}\u{1F1FF}");
    m.insert("flag_for_vanuatu", "\u{1F1FB}\u{1F1FA}");
    m.insert("flag_for_vatican_city", "\u{1F1FB}\u{1F1E6}");
    m.insert("flag_for_venezuela", "\u{1F1FB}\u{1F1EA}");
    m.insert("flag_for_vietnam", "\u{1F1FB}\u{1F1F3}");
    m.insert("flag_for_wallis_&_futuna", "\u{1F1FC}\u{1F1EB}");
    m.insert("flag_for_western_sahara", "\u{1F1EA}\u{1F1ED}");
    m.insert("flag_for_yemen", "\u{1F1FE}\u{1F1EA}");
    m.insert("flag_for_zambia", "\u{1F1FF}\u{1F1F2}");
    m.insert("flag_for_zimbabwe", "\u{1F1FF}\u{1F1FC}");
    m.insert("flag_for_åland_islands", "\u{1F1E6}\u{1F1FD}");
    m.insert("golf", "\u{26F3}");
    m.insert("fleur__de__lis", "\u{269C}");
    m.insert("muscle", "\u{1F4AA}");
    m.insert("flushed", "\u{1F633}");
    m.insert("frame_with_picture", "\u{1F5BC}");
    m.insert("fries", "\u{1F35F}");
    m.insert("frog", "\u{1F438}");
    m.insert("hatched_chick", "\u{1F425}");
    m.insert("frowning", "\u{1F626}");
    m.insert("fuelpump", "\u{26FD}");
    m.insert("full_moon_with_face", "\u{1F31D}");
    m.insert("gem", "\u{1F48E}");
    m.insert("star2", "\u{1F31F}");
    m.insert("golfer", "\u{1F3CC}");
    m.insert("mortar_board", "\u{1F393}");
    m.insert("grimacing", "\u{1F62C}");
    m.insert("smile_cat", "\u{1F638}");
    m.insert("grinning", "\u{1F600}");
    m.insert("grin", "\u{1F601}");
    m.insert("heartpulse", "\u{1F497}");
    m.insert("guardsman", "\u{1F482}");
    m.insert("haircut", "\u{1F487}");
    m.insert("hamster", "\u{1F439}");
    m.insert("raising_hand", "\u{1F64B}");
    m.insert("headphones", "\u{1F3A7}");
    m.insert("hear_no_evil", "\u{1F649}");
    m.insert("cupid", "\u{1F498}");
    m.insert("gift_heart", "\u{1F49D}");
    m.insert("heart", "\u{2764}");
    m.insert("exclamation", "\u{2757}");
    m.insert("heavy_exclamation_mark", "\u{2757}");
    m.insert("heavy_heart_exclamation_mark_ornament", "\u{2763}");
    m.insert("o", "\u{2B55}");
    m.insert("helm_symbol", "\u{2388}");
    m.insert("helmet_with_white_cross", "\u{26D1}");
    m.insert("high_heel", "\u{1F460}");
    m.insert("bullettrain_side", "\u{1F684}");
    m.insert("bullettrain_front", "\u{1F685}");
    m.insert("high_brightness", "\u{1F506}");
    m.insert("zap", "\u{26A1}");
    m.insert("hocho", "\u{1F52A}");
    m.insert("knife", "\u{1F52A}");
    m.insert("bee", "\u{1F41D}");
    m.insert("traffic_light", "\u{1F6A5}");
    m.insert("racehorse", "\u{1F40E}");
    m.insert("coffee", "\u{2615}");
    m.insert("hotsprings", "\u{2668}");
    m.insert("hourglass", "\u{231B}");
    m.insert("hourglass_flowing_sand", "\u{23F3}");
    m.insert("house_buildings", "\u{1F3D8}");
    m.insert("100", "\u{1F4AF}");
    m.insert("hushed", "\u{1F62F}");
    m.insert("ice_hockey_stick_and_puck", "\u{1F3D2}");
    m.insert("imp", "\u{1F47F}");
    m.insert("information_desk_person", "\u{1F481}");
    m.insert("information_source", "\u{2139}");
    m.insert("capital_abcd", "\u{1F520}");
    m.insert("abc", "\u{1F524}");
    m.insert("abcd", "\u{1F521}");
    m.insert("1234", "\u{1F522}");
    m.insert("symbols", "\u{1F523}");
    m.insert("izakaya_lantern", "\u{1F3EE}");
    m.insert("lantern", "\u{1F3EE}");
    m.insert("jack_o_lantern", "\u{1F383}");
    m.insert("dolls", "\u{1F38E}");
    m.insert("japanese_goblin", "\u{1F47A}");
    m.insert("japanese_ogre", "\u{1F479}");
    m.insert("beginner", "\u{1F530}");
    m.insert("zero", "0\u{FE0F}\u{20E3}");
    m.insert("one", "1\u{FE0F}\u{20E3}");
    m.insert("ten", "\u{1F51F}");
    m.insert("two", "2\u{FE0F}\u{20E3}");
    m.insert("three", "3\u{FE0F}\u{20E3}");
    m.insert("four", "4\u{FE0F}\u{20E3}");
    m.insert("five", "5\u{FE0F}\u{20E3}");
    m.insert("six", "6\u{FE0F}\u{20E3}");
    m.insert("seven", "7\u{FE0F}\u{20E3}");
    m.insert("eight", "8\u{FE0F}\u{20E3}");
    m.insert("nine", "9\u{FE0F}\u{20E3}");
    m.insert("couplekiss", "\u{1F48F}");
    m.insert("kissing_cat", "\u{1F63D}");
    m.insert("kissing", "\u{1F617}");
    m.insert("kissing_closed_eyes", "\u{1F61A}");
    m.insert("kissing_smiling_eyes", "\u{1F619}");
    m.insert("beetle", "\u{1F41E}");
    m.insert("large_blue_circle", "\u{1F535}");
    m.insert("last_quarter_moon_with_face", "\u{1F31C}");
    m.insert("leaves", "\u{1F343}");
    m.insert("mag", "\u{1F50D}");
    m.insert("left_right_arrow", "\u{2194}");
    m.insert("leftwards_arrow_with_hook", "\u{21A9}");
    m.insert("arrow_left", "\u{2B05}");
    m.insert("lock", "\u{1F512}");
    m.insert("lock_with_ink_pen", "\u{1F50F}");
    m.insert("sob", "\u{1F62D}");
    m.insert("low_brightness", "\u{1F505}");
    m.insert("lower_left_ballpoint_pen", "\u{1F58A}");
    m.insert("lower_left_crayon", "\u{1F58D}");
    m.insert("lower_left_fountain_pen", "\u{1F58B}");
    m.insert("lower_left_paintbrush", "\u{1F58C}");
    m.insert("mahjong", "\u{1F004}");
    m.insert("couple", "\u{1F46B}");
    m.insert("man_in_business_suit_levitating", "\u{1F574}");
    m.insert("man_with_gua_pi_mao", "\u{1F472}");
    m.insert("man_with_turban", "\u{1F473}");
    m.insert("mans_shoe", "\u{1F45E}");
    m.insert("shoe", "\u{1F45E}");
    m.insert("menorah_with_nine_branches", "\u{1F54E}");
    m.insert("mens", "\u{1F6B9}");
    m.insert("minidisc", "\u{1F4BD}");
    m.insert("iphone", "\u{1F4F1}");
    m.insert("calling", "\u{1F4F2}");
    m.insert("money__mouth_face", "\u{1F911}");
    m.insert("moneybag", "\u{1F4B0}");
    m.insert("rice_scene", "\u{1F391}");
    m.insert("mountain_bicyclist", "\u{1F6B5}");
    m.insert("mouse2", "\u{1F401}");
    m.insert("lips", "\u{1F444}");
    m.insert("moyai", "\u{1F5FF}");
    m.insert("notes", "\u{1F3B6}");
    m.insert("nail_care", "\u{1F485}");
    m.insert("ab", "\u{1F18E}");
    m.insert("negative_squared_cross_mark", "\u{274E}");
    m.insert("a", "\u{1F170}");
    m.insert("b", "\u{1F171}");
    m.insert("o2", "\u{1F17E}");
    m.insert("parking", "\u{1F17F}");
    m.insert("new_moon_with_face", "\u{1F31A}");
    m.insert("no_entry_sign", "\u{1F6AB}");
    m.insert("underage", "\u{1F51E}");
    m.insert("non__potable_water", "\u{1F6B1}");
    m.insert("arrow_upper_right", "\u{2197}");
    m.insert("arrow_upper_left", "\u{2196}");
    m.insert("office", "\u{1F3E2}");
    m.insert("older_man", "\u{1F474}");
    m.insert("older_woman", "\u{1F475}");
    m.insert("om_symbol", "\u{1F549}");
    m.insert("on", "\u{1F51B}");
    m.insert("book", "\u{1F4D6}");
    m.insert("unlock", "\u{1F513}");
    m.insert("mailbox_with_no_mail", "\u{1F4ED}");
    m.insert("mailbox_with_mail", "\u{1F4EC}");
    m.insert("cd", "\u{1F4BF}");
    m.insert("tada", "\u{1F389}");
    m.insert("feet", "\u{1F43E}");
    m.insert("walking", "\u{1F6B6}");
    m.insert("pencil2", "\u{270F}");
    m.insert("pensive", "\u{1F614}");
    m.insert("persevere", "\u{1F623}");
    m.insert("bow", "\u{1F647}");
    m.insert("raised_hands", "\u{1F64C}");
    m.insert("person_with_ball", "\u{26F9}");
    m.insert("person_with_blond_hair", "\u{1F471}");
    m.insert("pray", "\u{1F64F}");
    m.insert("person_with_pouting_face", "\u{1F64E}");
    m.insert("computer", "\u{1F4BB}");
    m.insert("pig2", "\u{1F416}");
    m.insert("hankey", "\u{1F4A9}");
    m.insert("poop", "\u{1F4A9}");
    m.insert("shit", "\u{1F4A9}");
    m.insert("bamboo", "\u{1F38D}");
    m.insert("gun", "\u{1F52B}");
    m.insert("black_joker", "\u{1F0CF}");
    m.insert("rotating_light", "\u{1F6A8}");
    m.insert("cop", "\u{1F46E}");
    m.insert("stew", "\u{1F372}");
    m.insert("pouch", "\u{1F45D}");
    m.insert("pouting_cat", "\u{1F63E}");
    m.insert("rage", "\u{1F621}");
    m.insert("put_litter_in_its_place", "\u{1F6AE}");
    m.insert("rabbit2", "\u{1F407}");
    m.insert("racing_motorcycle", "\u{1F3CD}");
    m.insert("radioactive_sign", "\u{2622}");
    m.insert("fist", "\u{270A}");
    m.insert("hand", "\u{270B}");
    m.insert("raised_hand_with_fingers_splayed", "\u{1F590}");
    m.insert(
        "raised_hand_with_part_between_middle_and_ring_fingers",
        "\u{1F596}",
    );
    m.insert("blue_car", "\u{1F699}");
    m.insert("apple", "\u{1F34E}");
    m.insert("relieved", "\u{1F60C}");
    m.insert("reversed_hand_with_middle_finger_extended", "\u{1F595}");
    m.insert("mag_right", "\u{1F50E}");
    m.insert("arrow_right_hook", "\u{21AA}");
    m.insert("sweet_potato", "\u{1F360}");
    m.insert("robot", "\u{1F916}");
    m.insert("rolled__up_newspaper", "\u{1F5DE}");
    m.insert("rowboat", "\u{1F6A3}");
    m.insert("runner", "\u{1F3C3}");
    m.insert("running", "\u{1F3C3}");
    m.insert("running_shirt_with_sash", "\u{1F3BD}");
    m.insert("boat", "\u{26F5}");
    m.insert("scales", "\u{2696}");
    m.insert("school_satchel", "\u{1F392}");
    m.insert("scorpius", "\u{264F}");
    m.insert("see_no_evil", "\u{1F648}");
    m.insert("sheep", "\u{1F411}");
    m.insert("stars", "\u{1F320}");
    m.insert("cake", "\u{1F370}");
    m.insert("six_pointed_star", "\u{1F52F}");
    m.insert("ski", "\u{1F3BF}");
    m.insert("sleeping_accommodation", "\u{1F6CC}");
    m.insert("sleeping", "\u{1F634}");
    m.insert("sleepy", "\u{1F62A}");
    m.insert("sleuth_or_spy", "\u{1F575}");
    m.insert("heart_eyes_cat", "\u{1F63B}");
    m.insert("smiley_cat", "\u{1F63A}");
    m.insert("innocent", "\u{1F607}");
    m.insert("heart_eyes", "\u{1F60D}");
    m.insert("smiling_imp", "\u{1F608}");
    m.insert("smiley", "\u{1F603}");
    m.insert("sweat_smile", "\u{1F605}");
    m.insert("smile", "\u{1F604}");
    m.insert("laughing", "\u{1F606}");
    m.insert("satisfied", "\u{1F606}");
    m.insert("blush", "\u{1F60A}");
    m.insert("smirk", "\u{1F60F}");
    m.insert("smoking", "\u{1F6AC}");
    m.insert("snow_capped_mountain", "\u{1F3D4}");
    m.insert("soccer", "\u{26BD}");
    m.insert("icecream", "\u{1F366}");
    m.insert("soon", "\u{1F51C}");
    m.insert("arrow_lower_right", "\u{2198}");
    m.insert("arrow_lower_left", "\u{2199}");
    m.insert("speak_no_evil", "\u{1F64A}");
    m.insert("speaker", "\u{1F508}");
    m.insert("mute", "\u{1F507}");
    m.insert("sound", "\u{1F509}");
    m.insert("loud_sound", "\u{1F50A}");
    m.insert("speaking_head_in_silhouette", "\u{1F5E3}");
    m.insert("spiral_calendar_pad", "\u{1F5D3}");
    m.insert("spiral_note_pad", "\u{1F5D2}");
    m.insert("shell", "\u{1F41A}");
    m.insert("sweat_drops", "\u{1F4A6}");
    m.insert("u5272", "\u{1F239}");
    m.insert("u5408", "\u{1F234}");
    m.insert("u55b6", "\u{1F23A}");
    m.insert("u6307", "\u{1F22F}");
    m.insert("u6708", "\u{1F237}");
    m.insert("u6709", "\u{1F236}");
    m.insert("u6e80", "\u{1F235}");
    m.insert("u7121", "\u{1F21A}");
    m.insert("u7533", "\u{1F238}");
    m.insert("u7981", "\u{1F232}");
    m.insert("u7a7a", "\u{1F233}");
    m.insert("cl", "\u{1F191}");
    m.insert("cool", "\u{1F192}");
    m.insert("free", "\u{1F193}");
    m.insert("id", "\u{1F194}");
    m.insert("koko", "\u{1F201}");
    m.insert("sa", "\u{1F202}");
    m.insert("new", "\u{1F195}");
    m.insert("ng", "\u{1F196}");
    m.insert("ok", "\u{1F197}");
    m.insert("sos", "\u{1F198}");
    m.insert("up", "\u{1F199}");
    m.insert("vs", "\u{1F19A}");
    m.insert("steam_locomotive", "\u{1F682}");
    m.insert("ramen", "\u{1F35C}");
    m.insert("partly_sunny", "\u{26C5}");
    m.insert("city_sunrise", "\u{1F307}");
    m.insert("surfer", "\u{1F3C4}");
    m.insert("swimmer", "\u{1F3CA}");
    m.insert("shirt", "\u{1F455}");
    m.insert("tshirt", "\u{1F455}");
    m.insert("table_tennis_paddle_and_ball", "\u{1F3D3}");
    m.insert("tea", "\u{1F375}");
    m.insert("tv", "\u{1F4FA}");
    m.insert("three_button_mouse", "\u{1F5B1}");
    m.insert("+1", "\u{1F44D}");
    m.insert("thumbsup", "\u{1F44D}");
    m.insert("__1", "\u{1F44E}");
    m.insert("-1", "\u{1F44E}");
    m.insert("thumbsdown", "\u{1F44E}");
    m.insert("thunder_cloud_and_rain", "\u{26C8}");
    m.insert("tiger2", "\u{1F405}");
    m.insert("tophat", "\u{1F3A9}");
    m.insert("top", "\u{1F51D}");
    m.insert("tm", "\u{2122}");
    m.insert("train2", "\u{1F686}");
    m.insert("triangular_flag_on_post", "\u{1F6A9}");
    m.insert("trident", "\u{1F531}");
    m.insert("twisted_rightwards_arrows", "\u{1F500}");
    m.insert("unamused", "\u{1F612}");
    m.insert("small_red_triangle", "\u{1F53A}");
    m.insert("arrow_up_small", "\u{1F53C}");
    m.insert("arrow_up_down", "\u{2195}");
    m.insert("upside__down_face", "\u{1F643}");
    m.insert("arrow_up", "\u{2B06}");
    m.insert("v", "\u{270C}");
    m.insert("vhs", "\u{1F4FC}");
    m.insert("wc", "\u{1F6BE}");
    m.insert("ocean", "\u{1F30A}");
    m.insert("waving_black_flag", "\u{1F3F4}");
    m.insert("wave", "\u{1F44B}");
    m.insert("waving_white_flag", "\u{1F3F3}");
    m.insert("moon", "\u{1F314}");
    m.insert("scream_cat", "\u{1F640}");
    m.insert("weary", "\u{1F629}");
    m.insert("weight_lifter", "\u{1F3CB}");
    m.insert("whale2", "\u{1F40B}");
    m.insert("wheelchair", "\u{267F}");
    m.insert("point_down", "\u{1F447}");
    m.insert("grey_exclamation", "\u{2755}");
    m.insert("white_frowning_face", "\u{2639}");
    m.insert("white_check_mark", "\u{2705}");
    m.insert("point_left", "\u{1F448}");
    m.insert("white_medium_small_square", "\u{25FD}");
    m.insert("star", "\u{2B50}");
    m.insert("grey_question", "\u{2754}");
    m.insert("point_right", "\u{1F449}");
    m.insert("relaxed", "\u{263A}");
    m.insert("white_sun_behind_cloud", "\u{1F325}");
    m.insert("white_sun_behind_cloud_with_rain", "\u{1F326}");
    m.insert("white_sun_with_small_cloud", "\u{1F324}");
    m.insert("point_up_2", "\u{1F446}");
    m.insert("point_up", "\u{261D}");
    m.insert("wind_blowing_face", "\u{1F32C}");
    m.insert("wink", "\u{1F609}");
    m.insert("wolf", "\u{1F43A}");
    m.insert("dancers", "\u{1F46F}");
    m.insert("boot", "\u{1F462}");
    m.insert("womans_clothes", "\u{1F45A}");
    m.insert("womans_hat", "\u{1F452}");
    m.insert("sandal", "\u{1F461}");
    m.insert("womens", "\u{1F6BA}");
    m.insert("worried", "\u{1F61F}");
    m.insert("gift", "\u{1F381}");
    m.insert("zipper__mouth_face", "\u{1F910}");
    m.insert("regional_indicator_a", "\u{1F1E6}");
    m.insert("regional_indicator_b", "\u{1F1E7}");
    m.insert("regional_indicator_c", "\u{1F1E8}");
    m.insert("regional_indicator_d", "\u{1F1E9}");
    m.insert("regional_indicator_e", "\u{1F1EA}");
    m.insert("regional_indicator_f", "\u{1F1EB}");
    m.insert("regional_indicator_g", "\u{1F1EC}");
    m.insert("regional_indicator_h", "\u{1F1ED}");
    m.insert("regional_indicator_i", "\u{1F1EE}");
    m.insert("regional_indicator_j", "\u{1F1EF}");
    m.insert("regional_indicator_k", "\u{1F1F0}");
    m.insert("regional_indicator_l", "\u{1F1F1}");
    m.insert("regional_indicator_m", "\u{1F1F2}");
    m.insert("regional_indicator_n", "\u{1F1F3}");
    m.insert("regional_indicator_o", "\u{1F1F4}");
    m.insert("regional_indicator_p", "\u{1F1F5}");
    m.insert("regional_indicator_q", "\u{1F1F6}");
    m.insert("regional_indicator_r", "\u{1F1F7}");
    m.insert("regional_indicator_s", "\u{1F1F8}");
    m.insert("regional_indicator_t", "\u{1F1F9}");
    m.insert("regional_indicator_u", "\u{1F1FA}");
    m.insert("regional_indicator_v", "\u{1F1FB}");
    m.insert("regional_indicator_w", "\u{1F1FC}");
    m.insert("regional_indicator_x", "\u{1F1FD}");
    m.insert("regional_indicator_y", "\u{1F1FE}");
    m.insert("regional_indicator_z", "\u{1F1FF}");
    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_map_not_empty() {
        assert!(!EMOJI.is_empty());
        assert_eq!(EMOJI.len(), 3608);
    }

    #[test]
    fn test_known_lookups() {
        assert_eq!(EMOJI.get("1st_place_medal"), Some(&"\u{1F947}"));
        assert_eq!(EMOJI.get("thumbs_up"), Some(&"\u{1F44D}"));
        assert_eq!(EMOJI.get("waving_hand"), Some(&"\u{1F44B}"));
        assert_eq!(EMOJI.get("heart"), Some(&"\u{2764}"));
        assert_eq!(EMOJI.get("smile"), Some(&"\u{1F604}"));
    }

    #[test]
    fn test_unknown_name_returns_none() {
        assert_eq!(EMOJI.get("this_emoji_does_not_exist_xyz"), None);
    }
}
