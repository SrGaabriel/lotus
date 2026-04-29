use cstree::build::Checkpoint;

use crate::{green::Parser, red::SyntaxKind};

pub struct Marker {
    checkpoint: Checkpoint,
    consumed: bool,
}

impl Marker {
    pub(crate) fn new(checkpoint: Checkpoint) -> Self {
        Self {
            checkpoint,
            consumed: false,
        }
    }

    pub fn complete(mut self, p: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
        self.consumed = true;
        p.start_node_at(self.checkpoint, kind);
        p.finish_node();
        CompletedMarker {
            checkpoint: self.checkpoint,
            kind,
        }
    }

    pub fn abandon(mut self, _p: &mut Parser) {
        self.consumed = true;
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        debug_assert!(
            self.consumed,
            "Marker dropped without complete/abandon — parser bug"
        );
    }
}

#[derive(Clone, Copy)]
pub struct CompletedMarker {
    checkpoint: Checkpoint,
    kind: SyntaxKind,
}

impl CompletedMarker {
    pub fn precede(self, _p: &mut Parser) -> Marker {
        Marker::new(self.checkpoint)
    }

    pub fn kind(self) -> SyntaxKind {
        self.kind
    }
}
