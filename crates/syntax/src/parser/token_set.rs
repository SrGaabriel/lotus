use crate::lexer::TokenKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenSet(u128);

impl TokenSet {
    pub const EMPTY: Self = Self(0);

    pub const fn new(kinds: &[TokenKind]) -> Self {
        let mut bits = 0u128;
        let mut i = 0;
        while i < kinds.len() {
            bits |= mask(kinds[i]);
            i += 1;
        }
        Self(bits)
    }

    pub const fn of(kind: TokenKind) -> Self {
        Self(mask(kind))
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, k: TokenKind) -> bool {
        self.0 & mask(k) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

const fn mask(k: TokenKind) -> u128 {
    1u128 << k.as_index()
}
