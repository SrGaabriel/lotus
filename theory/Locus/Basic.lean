import Std.Data.HashMap

open Std

inductive Level where
  | zero  : Level
  | succ  : Level → Level
  | max   : Level → Level → Level
  | imax  : Level → Level → Level
  | param : String → Level
  | mvar  : Nat → Level
  deriving Repr

instance : Inhabited Level where
  default := Level.zero

inductive Expr where
  | bvar  : Nat -> Expr
  | app   : Expr → Expr → Expr
  | lam   : Expr → Expr
  | pi    : Expr → Expr → Expr
  | sort  : Level -> Expr
  | const : Nat -> Expr

abbrev Context := List Expr
abbrev Env := Array Expr

def lift : Expr → Nat → Nat → Expr
  | .bvar n, k, d =>
      if n >= k then .bvar (n + d)
      else .bvar n
  | .app f x, k, d =>
      .app (lift f k d) (lift x k d)
  | .lam b, k, d =>
      .lam (lift b (k + 1) d)
  | .pi a b, k, d =>
      .pi (lift a k d) (lift b (k + 1) d)
  | .sort l, _, _ => .sort l
  | .const n, _, _ => .const n

def subst : Expr → Nat → Expr → Expr
  | .bvar n, k, s =>
      if n == k then s
      else if n > k then .bvar (n - 1)
      else .bvar n
  | .app f x, k, s =>
      .app (subst f k s) (subst x k s)
  | .lam b, k, s =>
      .lam (subst b (k + 1) (lift s 0 1))
  | .pi a b, k, s =>
      .pi (subst a k s) (subst b (k + 1) s)
  | .sort l, _, _ => .sort l
  | .const n, _, _ => .const n

inductive Reduce : Env → Expr → Expr → Prop
  | beta : Reduce env (.app (.lam b) e) (subst b 0 (lift e 0 1))
  | delta {env : Env} {n : Nat} {e : Expr} :
      env[n]? = some e →
      Reduce env (.const n) e

inductive ReduceStar : Env → Expr → Expr → Prop
  | refl {env : Env} {e : Expr} :
      ReduceStar env e e

  | step {env : Env} {e₁ e₂ e₃ : Expr} :
      Reduce env e₁ e₂ →
      ReduceStar env e₂ e₃ →
      ReduceStar env e₁ e₃

/-- Church-Rosser property for the reduction relation: if a term reduces to two different terms, then there is a common term they both reduce to -/
theorem reduce_star_confluent {env : Env} {e₁ e₂ e₃ : Expr} :
  ReduceStar env e₁ e₂ →
  ReduceStar env e₁ e₃ →
  ∃ e₄, ReduceStar env e₂ e₄ ∧ ReduceStar env e₃ e₄ := by
  intro h12 h13
  induction h12 generalizing e₃ with
  | refl =>
      exact ⟨e₃, by
        constructor
        · exact h13
        · exact ReduceStar.refl
      ⟩
  | step h12' h23 ih =>
    cases h13 with
      | refl =>
          exact ⟨_, by
            constructor
            · exact ReduceStar.refl
            · exact ReduceStar.step h12' h23
          ⟩
      | step h13' h33 =>
          cases h12' with
          | beta =>
              cases h13' with
              | beta =>
                  exact ih h33
          | delta h1 =>
              cases h13' with
              | delta h2 =>
                  cases h1.symm.trans h2
                  exact ih h33
