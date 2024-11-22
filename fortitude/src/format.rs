use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    process::ExitCode,
};

use crate::{
    check::get_files,
    cli::{FormatArgs, FORTRAN_EXTS},
};

use anyhow::Result;
use itertools::Itertools;
use topiary_core::{formatter, FormatterError, Language, Operation, TopiaryQuery};

fn topiary_query() -> &'static str {
    include_str!("../resources/format/fortran.scm")
}

pub fn format(args: FormatArgs) -> Result<ExitCode> {
    let files = &args.files.unwrap_or_default();
    let file_extensions = &args
        .file_extensions
        .unwrap_or(FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect_vec());

    let grammar: topiary_tree_sitter_facade::Language = tree_sitter_fortran::LANGUAGE.into();
    let query = TopiaryQuery::new(&grammar, topiary_query()).expect("building topiary query");
    let language = Language {
        name: "fortran".to_string(),
        query,
        grammar,
        indent: None,
    };

    for file in get_files(files, file_extensions) {
        match format_file(file, &language) {
            Ok(_) => continue,
            Err(err) => {
                println!("Formatter error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}

fn format_file(file: PathBuf, language: &Language) -> Result<(), FormatterError> {
    println!("formatting {file:?}");
    let input = File::open(file)?;

    let output = std::io::stdout();
    let mut buf_input = BufReader::new(input);
    let mut buf_output = BufWriter::new(output);

    formatter(
        &mut buf_input,
        &mut buf_output,
        language,
        // TODO: user args?
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: true,
        },
    )?;

    buf_output.into_inner()?;
    Ok(())
}
