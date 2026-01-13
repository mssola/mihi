use mihi::{group_declension_inflections, Category, DeclensionInfo, DeclensionTable, Gender, Word};

fn get_inflected_from(word: &Word, row: &[DeclensionInfo; 2]) -> String {
    if word.is_flag_set("onlysingular") {
        row[0].inflected.join("/")
    } else if word.is_flag_set("onlyplural") {
        row[1].inflected.join("/")
    } else {
        format!(
            "{}, {}",
            row[0].inflected.join("/"),
            row[1].inflected.join("/")
        )
    }
}

fn get_noun_table(word: &Word) -> Result<DeclensionTable, String> {
    let gender = match word.gender {
        Gender::MasculineOrFeminine => Gender::Masculine as usize,
        _ => word.gender as usize,
    };
    group_declension_inflections(word, &word.kind, gender)
}

fn print_noun_inflection(word: &Word) -> Result<(), String> {
    let table = get_noun_table(word)?;

    println!("\n== Inflection ==\n");

    println!(
        "Nominative:\t{}",
        get_inflected_from(word, &table.nominative)
    );
    println!("Vocative:\t{}", get_inflected_from(word, &table.vocative));
    println!(
        "Accusative:\t{}",
        get_inflected_from(word, &table.accusative)
    );
    println!("Genitive:\t{}", get_inflected_from(word, &table.genitive));
    println!("Dative:\t\t{}", get_inflected_from(word, &table.dative));
    println!("Ablative:\t{}", get_inflected_from(word, &table.ablative));
    if word.locative {
        println!("Locative:\t{}", get_inflected_from(word, &table.locative));
    }

    Ok(())
}

fn get_adjective_table(word: &Word) -> Result<[DeclensionTable; 3], String> {
    // Unless the word is a special "unus nauta" variant, force 1/2 declension
    // adjectives in the feminine to grab the "a" kind.
    let kind_f = if word.kind.as_str() == "unusnauta" {
        &word.kind
    } else {
        match word.declension_id {
            Some(1 | 2) => &"a".to_string(),
            _ => &word.kind,
        }
    };

    let kind_n = if word.kind == "us" {
        &"um".to_owned()
    } else {
        &word.kind
    };

    Ok([
        group_declension_inflections(word, &word.kind, Gender::Masculine as usize)?,
        group_declension_inflections(word, kind_f, Gender::Feminine as usize)?,
        group_declension_inflections(word, kind_n, Gender::Neuter as usize)?,
    ])
}

fn print_adjective_inflection(word: &Word) -> Result<(), String> {
    let tables = get_adjective_table(word)?;

    println!("\n== Inflection ==\n");

    println!(
        "Nominative:\t{} | {} | {}",
        get_inflected_from(word, &tables[0].nominative),
        get_inflected_from(word, &tables[1].nominative),
        get_inflected_from(word, &tables[2].nominative)
    );
    println!(
        "Vocative:\t{} | {} | {}",
        get_inflected_from(word, &tables[0].vocative),
        get_inflected_from(word, &tables[1].vocative),
        get_inflected_from(word, &tables[2].vocative)
    );
    println!(
        "Accusative:\t{} | {} | {}",
        get_inflected_from(word, &tables[0].accusative),
        get_inflected_from(word, &tables[1].accusative),
        get_inflected_from(word, &tables[2].accusative)
    );
    println!(
        "Genitive:\t{} | {} | {}",
        get_inflected_from(word, &tables[0].genitive),
        get_inflected_from(word, &tables[1].genitive),
        get_inflected_from(word, &tables[2].genitive)
    );
    println!(
        "Dative:\t\t{} | {} | {}",
        get_inflected_from(word, &tables[0].dative),
        get_inflected_from(word, &tables[1].dative),
        get_inflected_from(word, &tables[2].dative)
    );
    println!(
        "Ablative:\t{} | {} | {}",
        get_inflected_from(word, &tables[0].ablative),
        get_inflected_from(word, &tables[1].ablative),
        get_inflected_from(word, &tables[2].ablative)
    );
    if word.locative {
        println!(
            "Locative:\t{} | {} | {}",
            get_inflected_from(word, &tables[0].locative),
            get_inflected_from(word, &tables[1].locative),
            get_inflected_from(word, &tables[2].locative)
        );
    }

    Ok(())
}

pub fn print_full_inflection_for(word: Word) -> Result<(), String> {
    if word.is_flag_set("indeclinable") {
        return Ok(());
    }

    match word.category {
        Category::Noun => print_noun_inflection(&word)?,
        Category::Adjective => print_adjective_inflection(&word)?,
        Category::Verb => {}    // TODO
        Category::Pronoun => {} // TODO
        Category::Adverb
        | Category::Preposition
        | Category::Conjunction
        | Category::Interjection
        | Category::Determiner
        | Category::Unknown => {
            // Nothing to do.
        }
    }
    // TODO: on the 'extra' info.

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_word(enunciated: &str) -> Word {
        let words = mihi::select_enunciated(Some(enunciated.to_string())).unwrap();

        assert_eq!(words.len(), 1);

        mihi::find_by(words.first().unwrap().as_str()).unwrap()
    }

    fn stringify_with(word: &Word, table: &DeclensionTable) -> String {
        let mut res = get_inflected_from(word, &table.nominative);
        res.push_str(" | ");
        res.push_str(get_inflected_from(word, &table.vocative).as_str());
        res.push_str(" | ");
        res.push_str(get_inflected_from(word, &table.accusative).as_str());
        res.push_str(" | ");
        res.push_str(get_inflected_from(word, &table.genitive).as_str());
        res.push_str(" | ");
        res.push_str(get_inflected_from(word, &table.dative).as_str());
        res.push_str(" | ");
        res.push_str(get_inflected_from(word, &table.ablative).as_str());
        if word.locative {
            res.push_str(" | ");
            res.push_str(get_inflected_from(word, &table.locative).as_str());
        }

        res
    }

    fn assert_noun_table(enunciated: &str, expected: &str) {
        let word = get_word(enunciated);
        let table = get_noun_table(&word).unwrap();

        let res = stringify_with(&word, &table);

        assert_eq!(res, expected);
    }

    fn assert_adjective_table(enunciated: &str, masculine: &str, feminine: &str, neuter: &str) {
        let word = get_word(enunciated);
        let tables = get_adjective_table(&word).unwrap();

        let res = stringify_with(&word, &tables[0]);
        assert_eq!(res, masculine);

        let res = stringify_with(&word, &tables[1]);
        assert_eq!(res, feminine);

        let res = stringify_with(&word, &tables[2]);
        assert_eq!(res, neuter);
    }

    #[test]
    fn test_nouns() {
        assert_noun_table(
            "rosa, rosae",
            "rosa, rosae | rosa, rosae | rosam, rosās | rosae, rosārum | rosae, rosīs | rosā, rosīs",
        );
        assert_noun_table(
            "fīlia, fīliae",
            "fīlia, fīliae | fīlia, fīliae | fīliam, fīliās | fīliae, fīliārum | fīliae, fīliīs/fīliābus | fīliā, fīliīs/fīliābus",
        );
        assert_noun_table(
            "dea, deae",
            "dea, deae | dea, deae | deam, deās | deae, deārum | deae, deābus | deā, deābus",
        );
        assert_noun_table(
            "Rōma, Rōmae",
            "Rōma | Rōma | Rōmam | Rōmae | Rōmae | Rōmā | Rōmae",
        );
        assert_noun_table(
            "lupus, lupī",
            "lupus, lupī | lupe, lupī | lupum, lupōs | lupī, lupōrum | lupō, lupīs | lupō, lupīs",
        );
        assert_noun_table(
            "templum, templī",
            "templum, templa | templum, templa | templum, templa | templī, templōrum | templō, templīs | templō, templīs",
        );
        assert_noun_table(
            "vir, virī",
            "vir, virī | vir, virī | virum, virōs | virī, virōrum | virō, virīs | virō, virīs",
        );
        assert_noun_table(
            "liber, librī",
            "liber, librī | liber, librī | librum, librōs | librī, librōrum | librō, librīs | librō, librīs",
        );
        assert_noun_table(
            "fīlius, fīliī",
            "fīlius, fīliī | fīlī, fīliī | fīlium, fīliōs | fīlī/fīliī, fīliōrum | fīliō, fīliīs | fīliō, fīliīs",
        );
        assert_noun_table(
            "leō, leōnis",
            "leō, leōnēs | leō, leōnēs | leōnem, leōnēs | leōnis, leōnum | leōnī, leōnibus | leōne, leōnibus",
        );
        assert_noun_table(
            "ovis, ovis",
            "ovis, ovēs | ovis, ovēs | ovem, ovēs | ovis, ovium | ovī, ovibus | ove, ovibus",
        );
        assert_noun_table(
            "mare, maris",
            "mare, maria | mare, maria | mare, maria | maris, marium/marum | marī, maribus | marī/mare, maribus",
        );
        assert_noun_table(
            "Iuppiter, Iovis",
            "Iuppiter | Iuppiter | Iovem | Iovis | Iovī | Iove",
        );
        assert_noun_table(
            "portus, portūs",
            "portus, portūs | portus, portūs | portum, portūs | portūs, portuum | portuī, portibus | portū, portibus",
        );
        assert_noun_table(
            "cornū, cornūs",
            "cornū, cornua | cornū, cornua | cornū, cornua | cornūs, cornuum | cornuī, cornibus | cornū, cornibus",
        );
        // TODO: domus
        assert_noun_table(
            "diēs, diēī",
            "diēs, diēs | diēs, diēs | diem, diēs | diēī, diērum | diēī, diēbus | diē, diēbus",
        );
        assert_noun_table(
            "rēs, reī",
            "rēs, rēs | rēs, rēs | rem, rēs | reī, rērum | reī, rēbus | rē, rēbus",
        );
    }

    #[test]
    fn test_adjectives() {
        assert_adjective_table(
            "novus, nova, novum",
            "novus, novī | nove, novī | novum, novōs | novī, novōrum | novō, novīs | novō, novīs",
            "nova, novae | nova, novae | novam, novās | novae, novārum | novae, novīs | novā, novīs",
            "novum, nova | novum, nova | novum, nova | novī, novōrum | novō, novīs | novō, novīs",
        );
        assert_adjective_table(
            "pulcher, pulchra, pulchrum",
            "pulcher, pulchrī | pulcher, pulchrī | pulchrum, pulchrōs | pulchrī, pulchrōrum | pulchrō, pulchrīs | pulchrō, pulchrīs",
            "pulchra, pulchrae | pulchra, pulchrae | pulchram, pulchrās | pulchrae, pulchrārum | pulchrae, pulchrīs | pulchrā, pulchrīs",
            "pulcher, pulchra | pulcher, pulchra | pulcher, pulchra | pulchrī, pulchrōrum | pulchrō, pulchrīs | pulchrō, pulchrīs",
        );
        assert_adjective_table(
            "ūnus, ūna, ūnum",
            "ūnus, ūnī | ūne, ūnī | ūnum, ūnōs | ūnīus, ūnōrum | ūnī, ūnīs | ūnō, ūnīs",
            "ūna, ūnae | ūna, ūnae | ūnam, ūnās | ūnīus, ūnārum | ūnī, ūnīs | ūnā, ūnīs",
            "ūnum, ūna | ūnum, ūna | ūnum, ūna | ūnīus, ūnōrum | ūnī, ūnīs | ūnō, ūnīs",
        );
        assert_adjective_table(
            "ferōx, ferōx",
            "ferōx, ferōcēs | ferōx, ferōcēs | ferōcem, ferōcēs | ferōcis, ferōcium | ferōcī, ferōcibus | ferōcī, ferōcibus",
            "ferōx, ferōcēs | ferōx, ferōcēs | ferōcem, ferōcēs | ferōcis, ferōcium | ferōcī, ferōcibus | ferōcī, ferōcibus",
            "ferōx, ferōcia | ferōx, ferōcia | ferōx, ferōcia | ferōcis, ferōcium | ferōcī, ferōcibus | ferōcī, ferōcibus",
        );
        assert_adjective_table(
            "gravis, grave",
            "gravis, gravēs | gravis, gravēs | gravem/gravīs, gravēs | gravis, gravium | gravī, gravibus | gravī, gravibus",
            "gravis, gravēs | gravis, gravēs | gravem/gravīs, gravēs | gravis, gravium | gravī, gravibus | gravī, gravibus",
            "grave, gravia | grave, gravia | grave, gravia | gravis, gravium | gravī, gravibus | gravī, gravibus",
        );
        assert_adjective_table(
            "celer, celeris, celere",
            "celer, celerēs | celer, celerēs | celerem, celerēs | celeris, celerium | celerī, celeribus | celerī, celeribus",
            "celeris, celerēs | celeris, celerēs | celerem, celerēs | celeris, celerium | celerī, celeribus | celerī, celeribus",
            "celere, celeria | celere, celeria | celere, celeria | celeris, celerium | celerī, celeribus | celerī, celeribus"
        );
        assert_adjective_table(
            "duo, duae, duo",
            "duo | duo | duo/duōs | duōrum | duōbus | duōbus",
            "duae | duae | duās | duārum | duābus | duābus",
            "duo | duo | duo | duōrum | duōbus | duōbus",
        );
        assert_adjective_table(
            "trēs, trēs, tria",
            "trēs | trēs | trēs/trīs | trium | tribus | tribus",
            "trēs | trēs | trēs/trīs | trium | tribus | tribus",
            "tria | tria | tria | trium | tribus | tribus",
        );
        assert_adjective_table(
            "mīlle, mīlle",
            "mīlle, mīlia | mīlle, mīlia | mīlle, mīlia | mīlle, mīlium | mīlle, mīlibus | mīlle, mīlibus",
            "mīlle, mīlia | mīlle, mīlia | mīlle, mīlia | mīlle, mīlium | mīlle, mīlibus | mīlle, mīlibus",
            "mīlle, mīlia | mīlle, mīlia | mīlle, mīlia | mīlle, mīlium | mīlle, mīlibus | mīlle, mīlibus"
        );
    }
}
