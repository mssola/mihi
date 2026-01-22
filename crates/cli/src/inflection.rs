use mihi::{
    configuration, get_adjective_table, get_inflected_from, get_noun_table, Category, Word,
};

fn print_noun_inflection(word: &Word) -> Result<(), String> {
    let table = get_noun_table(word)?;

    println!("\n== Inflection ==\n");

    for idx in configuration().case_order.to_usizes() {
        match idx {
            0 => println!(
                "Nominative:\t{}",
                get_inflected_from(word, &table.nominative)
            ),
            1 => println!("Vocative:\t{}", get_inflected_from(word, &table.vocative)),
            2 => println!(
                "Accusative:\t{}",
                get_inflected_from(word, &table.accusative)
            ),
            3 => println!("Genitive:\t{}", get_inflected_from(word, &table.genitive)),
            4 => println!("Dative:\t\t{}", get_inflected_from(word, &table.dative)),
            5 => println!("Ablative:\t{}", get_inflected_from(word, &table.ablative)),
            6 => {
                if word.locative {
                    println!("Locative:\t{}", get_inflected_from(word, &table.locative));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn print_adjective_inflection(word: &Word) -> Result<(), String> {
    let tables = get_adjective_table(word)?;

    println!("\n== Inflection ==\n");

    for idx in configuration().case_order.to_usizes() {
        match idx {
            0 => println!(
                "Nominative:\t{} | {} | {}",
                get_inflected_from(word, &tables[0].nominative),
                get_inflected_from(word, &tables[1].nominative),
                get_inflected_from(word, &tables[2].nominative)
            ),
            1 => println!(
                "Vocative:\t{} | {} | {}",
                get_inflected_from(word, &tables[0].vocative),
                get_inflected_from(word, &tables[1].vocative),
                get_inflected_from(word, &tables[2].vocative)
            ),
            2 => println!(
                "Accusative:\t{} | {} | {}",
                get_inflected_from(word, &tables[0].accusative),
                get_inflected_from(word, &tables[1].accusative),
                get_inflected_from(word, &tables[2].accusative)
            ),
            3 => println!(
                "Genitive:\t{} | {} | {}",
                get_inflected_from(word, &tables[0].genitive),
                get_inflected_from(word, &tables[1].genitive),
                get_inflected_from(word, &tables[2].genitive)
            ),
            4 => println!(
                "Dative:\t\t{} | {} | {}",
                get_inflected_from(word, &tables[0].dative),
                get_inflected_from(word, &tables[1].dative),
                get_inflected_from(word, &tables[2].dative)
            ),
            5 => println!(
                "Ablative:\t{} | {} | {}",
                get_inflected_from(word, &tables[0].ablative),
                get_inflected_from(word, &tables[1].ablative),
                get_inflected_from(word, &tables[2].ablative)
            ),
            6 => {
                if word.locative {
                    println!(
                        "Locative:\t{} | {} | {}",
                        get_inflected_from(word, &tables[0].locative),
                        get_inflected_from(word, &tables[1].locative),
                        get_inflected_from(word, &tables[2].locative)
                    );
                }
            }
            _ => {}
        }
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
    use mihi::DeclensionTable;

    fn get_word(enunciated: &str) -> Word {
        let words = mihi::select_enunciated(Some(enunciated.to_string()), &[]).unwrap();

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
            "ovis, ovēs | ovis, ovēs | ovem, ovēs/ovīs | ovis, ovium | ovī, ovibus | ove, ovibus",
        );
        assert_noun_table(
            "turris, turris",
            "turris, turrēs | turris, turrēs | turrem/turrim, turrēs/turrīs | turris, turrium | turrī, turribus | turre/turrī, turribus",
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
