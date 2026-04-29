use std::ops::Range;

use ariadne::{ColorGenerator, Label as ReportLabel, Report, ReportKind};
use db::SourceFile;

use crate::{Diagnostic, Label, Severity, files::FilesCache};

pub fn render(cache: &mut FilesCache, diagnostic: &Diagnostic) {
    let report = to_report(diagnostic);
    report.print(cache).unwrap();
}

pub fn to_report(diagnostic: &Diagnostic) -> Report<'static, (SourceFile, Range<usize>)> {
    let mut colors = ColorGenerator::new();
    let kind = match diagnostic.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
        Severity::Lint => ReportKind::Advice,
    };
    let span = label_span(&diagnostic.primary);
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
        let span = label_span(label);
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

fn label_span(label: &Label) -> (SourceFile, Range<usize>) {
    let start = label.range.start().into();
    let end = label.range.end().into();
    (label.file, start..end)
}
