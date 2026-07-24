use crate::{
    Diagnostic,
    Label,
    Severity,
    files::FilesCache,
};
use ariadne::{
    Color,
    Config,
    Label as ReportLabel,
    Report,
    ReportKind,
};
use db::SourceFile;
use std::ops::Range;

const ARIADNE_CONFIG: Config = Config::new();
const SECONDARY_COLOR: Color = Color::Blue;

pub fn render(cache: &mut FilesCache, diagnostic: &Diagnostic) {
    let report = to_report(diagnostic);
    report.print(cache).unwrap();
}

pub fn to_report(diagnostic: &Diagnostic) -> Report<(SourceFile, Range<usize>)> {
    let primary_color = match diagnostic.severity {
        Severity::Error => Color::Red,
        Severity::Warning => Color::Yellow,
        Severity::Lint => Color::Cyan,
    };
    let kind = report_kind(diagnostic);
    let span = label_span(&diagnostic.primary);
    let mut report = Report::build(kind, span.clone())
        .with_config(ARIADNE_CONFIG)
        .with_message(&diagnostic.message);

    let mut primary_label = ReportLabel::new(span).with_color(primary_color);
    if let Some(message) = &diagnostic.primary.message {
        primary_label = primary_label.with_message(message);
    }
    report = report.with_label(primary_label);

    for label in &diagnostic.secondaries {
        let span = label_span(label);
        let mut report_label = ReportLabel::new(span).with_color(SECONDARY_COLOR);
        if let Some(message) = &label.message {
            report_label = report_label.with_message(message);
        }
        report = report.with_label(report_label);
    }
    if !diagnostic.notes.is_empty() {
        report.set_note(diagnostic.notes.join("\n      "));
    }
    if !diagnostic.helps.is_empty() {
        report.set_help(diagnostic.helps.join("\n      "));
    }
    report.finish()
}

fn report_kind(diagnostic: &Diagnostic) -> ReportKind {
    let (label, color) = match diagnostic.severity {
        Severity::Error => ("error", Color::Red),
        Severity::Warning => ("warning", Color::Yellow),
        Severity::Lint => ("lint", Color::Blue),
    };
    let name: &'static str = match diagnostic.code {
        Some(code) => Box::leak(format!("{label}[{code}]").into_boxed_str()),
        None => label,
    };
    ReportKind::Custom(name, color)
}

fn label_span(label: &Label) -> (SourceFile, Range<usize>) {
    let start = label.range.start().into();
    let end = label.range.end().into();
    (label.file, start..end)
}
