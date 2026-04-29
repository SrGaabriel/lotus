use std::{ops::Range, path::PathBuf};

use ariadne::{ColorGenerator, Label as ReportLabel, Report, ReportKind};

use crate::{
    Diagnostic, Label, Severity,
    files::{Files, FilesCache},
};

pub fn render(cache: &mut FilesCache, diagnostic: &Diagnostic) {
    let report = to_report(cache.files, diagnostic);
    report.eprint(cache).unwrap();
}

pub fn to_report<'a>(
    files: &'a Files,
    diagnostic: &Diagnostic,
) -> Report<'a, (PathBuf, Range<usize>)> {
    let mut colors = ColorGenerator::new();
    let kind = match diagnostic.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
        Severity::Lint => ReportKind::Advice,
    };
    let span = get_label_span(files, &diagnostic.primary);
    let mut report = Report::build(kind, span.clone()).with_message(&diagnostic.message);

    if let Some(code) = diagnostic.code {
        report = report.with_code(code);
    }
    let mut primary_label = ReportLabel::new(span).with_color(colors.next());
    if let Some(message) = &diagnostic.primary.message {
        primary_label = primary_label.with_message(message);
    }
    report = report.with_label(primary_label);

    for label in &diagnostic.secondaries {
        let span = get_label_span(files, label);
        let mut report_label = ReportLabel::new(span).with_color(colors.next());
        if let Some(message) = &label.message {
            report_label = report_label.with_message(message);
        }
        report = report.with_label(report_label);
    }
    if let Some(note) = diagnostic.notes.first() {
        report = report.with_note(note);
    }
    if let Some(help) = diagnostic.helps.first() {
        report = report.with_help(help);
    }
    report.finish()
}

fn get_label_span(files: &Files, label: &Label) -> (PathBuf, Range<usize>) {
    let path = files.get(label.file).path.clone();
    let start = label.range.start().into();
    let end = label.range.end().into();
    (path, start..end)
}
