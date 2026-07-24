#set page(
  margin: 1in,
)

#set text(
  font: "Times New Roman",
  size: 12pt,
)

#set heading(numbering: "1.")

#align(center)[
  #heading(level: 1, numbering: none)[The Road to Lotus]
  Gabriel Di Lucca Minatel
]

#heading(level: 1, numbering: none)[
  Introduction
]

This is a primer for the concepts and motivations behind Lotus, a dependently typed programming language. We will explore everything from the foundations of computation to the intricacies of type theory and logic.

#heading(level: 1)[
  What Programs Are
]

#heading(level: 2)[
  Models of Computation
]

There exist two models of computation we'll focus on: the Turing machine and the lambda calculus. The Turing machine is a theoretical model that consists of a tape divided into cells, a head that can read and write symbols on the tape and a state register that holds the current state of the machine. Everyone is familiar with the Turing machine, as it is the most widely used model of computation and is often used to define what it means for a function to be computable.

The lambda calculus, on the other hand, is a bit less known. It is a formal system for expressing computation that consists of exactly three constructs: variables, function definitions and function applications. It doesn't have a notion of state, memory, time, strings, numbers, etc. It is a purely functional model of computation where everything is a function and computation is done by applying functions to arguments.

Both are equivalent in terms of computational power, they are both Turing-Complete, which means that they can compute any function that is computable and can encode each other. You may find this surprising and ask "how can I write a text editor without strings?". You are already doing it, your CPU doesn't have a notion of strings, it operates only on bits. But we made it interpret certain combinations of bits as strings, numbers, etc. You can do the same with the lambda calculus.

The lambda calculus is a more abstract model of computation that focuses on the notion of functions and their application, while the Turing machine is a more concrete model that focuses on the notion of state and memory.

The imperative model is considered more efficient because our computers are designed to operate on a von Neumann architecture, which is based on the Turing machine. The lambda calculus, on the other hand, is more elegant and easier to mathematically reason about, which is why it is often used in the study of programming languages and type systems. That is because 99% of the field of logic is immutable and stateless, which is a perfect fit for the lambda calculus.

#heading(level: 2)[
  What is a Program?
]

In the Java/C/Python lineage, a program is fundamentally a sequence of instructions that mutate state over time. This is called the imperative paradigm, where programs are scripts that tell the machine what to do at each tick. A method is a procedure that takes inputs, possibly reads or writes shared state, possibly performs I/O, possibly throws, possibly exits or eventually returns.

Types in this world are mostly a tagging system for memory layouts and a sanity check on method calls, they simply describe what kind of bits sit at an address.

The meaning of a program under this philosophy is "what it does when you run it". This is called *operational semantics*. When reasoning about programs under this view, we are basically running a mental simulation of the machine executing the program. This is very intuitive and is how most programmers think about their code, however, it has some drawbacks. Reasoning about the behavior of a program without actually running it is hard, especially when the program has side effects or is non-deterministic. Proving correctness is even harder, since you have to consider all possible states and inputs that the program can encounter.

In the lambda calculus lineage, programs are just expressions that denote values. For example, given `factorial :: Int -> Int`, the expression `factorial 5` doesn't denote the process of calculating the factorial of 5, it denotes the value 120 in the same way that `2 + 2` denotes the value 4.

To execute a program in this world, we need to evaluate the expression until we get a value. This descends from the lambda calculus where computation is just the process of reducing an expression to its normal form.

This is called *denotational semantics*, since we purely reason about what our expressions denote and how they compose. This model gives us *referential transparency*, meaning that we can replace an expression with its value without changing the meaning of the program. We call this paradigm *functional programming*, since we focus on the composition of pure functions that take inputs and produce outputs without side effects. This makes it easier to reason about our code, since we don't have to worry about the state of the world or the order of execution.

So how do we execute side-effectful programs in pure functional programming? The answer is *monads*, but we will get to that later.

#heading(level: 2)[
  Lambda calculus as a programming language
]

We will quickly go over how to write programs in the lambda calculus, since it is the basis of all functional programming languages and, incidentally, of Lotus.

The syntax is very simple, variables are just identifiers ($x$, $y$, $z$, etc) and function applications/calls are written like $f thin x$ (means "apply function `f` to argument `x`"). The only other construct is function definitions, which are written as $lambda x. e$, meaning "function that takes an argument `x` and returns the expression `e`". Function application is left associative ($f thin x thin y$ is parsed as $(f thin x) thin y$) and function definitions are right associative ($lambda x. lambda y. e$ is parsed as $lambda x. (lambda y. e)$).

Functions don't have names (anonymous) in the lambda calculus and are expressions just like any other. They also only take one argument, but we can encode multiple arguments using currying.

Currying is the process of transforming a function that takes multiple arguments into a sequence of functions that each take a single argument. For example, a function that takes two arguments `f(x, y)` can be curried into `f(x)(y)`. So for example, $lambda x. lambda y. x + y$ is a curried function that takes two arguments `x` and `y` and returns their sum. If I call it only once with the number 5, I get back $lambda y. 5 + y$, and if I call it again with the number 3, I get back `5 + 3`, which reduces to `8`.

When analyzing a $lambda$-term, we can distinguish between free and bound variables. A variable is free if it is not introduced by any binder in the expression and bound otherwise. You may think that all variables in a lambda expression are bound and $x$ should throw an error in $lambda y. x + y$. But this is only true when we are analyzing the expression in isolation. When we analyze a lambda expression, we need to consider the context in which it appears. The full expression is actually $lambda x. lambda y. x + y$, where the first binder introduces `x` and the second binder introduces `y`. In this expression, `x` is free in the body of the second lambda, but it is bound in the context of the whole expression.

There are two main notions of equality:
1. *$alpha$-conversion*: renaming variables. For example, $lambda x. x$ is the same as $lambda y. y$.
2. *$beta$-reduction*: applying a function to an argument. For example, $(lambda x. x + 1) 5$ reduces to $5 + 1$.

*Note:* in these examples, we are using number literals and the addition operator for clarity, but in the pure lambda calculus, we would have to encode numbers and addition using functions as well.

Two processes that are often confused are substitution and reduction. Substitution is the process of replacing a variable with an expression, while reduction is the process of applying a function to an argument. There are many different notations for substitution you may encounter, such as $M[x := N]$, $M[N\/x]$ or $M\{x arrow.l N\}$, but they all mean "replace all free occurrences of `x` in `M` with `N`". Reduction, on the other hand, is the process of applying a function to an argument, which involves substituting the argument into the body of the function.

So how do we represent numbers? We use Church numerals, which are higher-order functions that take a function `f` and an argument `x` and apply `f` to `x` a certain number of times. For example, the Church numeral for 0 is $lambda f. lambda x. x$, which means "apply `f` zero times to `x`". The Church numeral for 1 is $lambda f. lambda x. f x$, which means "apply `f` once to `x`". The Church numeral for 2 is $lambda f. lambda x. f (f x)$, which means "apply `f` twice to `x`", etc.

So the function `succ` (successor) can be defined as $lambda n. lambda f. lambda x. n (f x)$, which takes a Church numeral `n` and returns a new Church numeral that applies `f` one more time than `n` does.

Church booleans are defined as follows: `true` is $lambda t. lambda f. t$, which means "choose the first argument" and `false` is $lambda t. lambda f. f$, which means "choose the second argument". There are many other encodings for data structures, such as pairs, lists, trees, etc. The point is that we can represent any data structure we want using just functions.

In the lambda calculus, computation is just the process of reducing redexes (reducible expressions) to its normal form (a value that cannot be reduced any further). For example, if we have the expression $(lambda x. x + 1) 5$, we can reduce it to $5 + 1$ and then to $6$. The process of reduction is confluent, meaning that the order in which we reduce expressions does not affect the final result.

We covered the untyped lambda calculus, but there are also typed versions of the lambda calculus, which introduce a type system to classify terms and ensure certain properties about them.

#heading(level: 1)[
  Types
]

#heading(level: 2)[
  Types as bookkeeping
]

In the imperative paradigm, a type is a label on bits. `int` signifies that the next four bytes should be read as a two's-complement integer, `String` represents a pointer to a heap-allocated sequence of characters, etc. The type system's job is to make sure you don't read those bytes the wrong way and that you don't conflate different kinds of data.

This is a useful job as it catches a lot of bugs at compile time and types describe layout and prevent confusion. However, they do not say much about what the program means. For example, a function with type `int -> int` could be either the factorial or a function that erases your hard drive and returns 0.

The internet is full of debates on static vs dynamic typing, but they are mostly arguing about the ergonomics of the bookkeeping system, mainly whether the safety checks are worth the extra annotations and development time.

The deeper question, however, about what a type is and what it means is not really on the table. The next section will show that there is a much richer reading of what a type is and this philosophy determines how we write and reason about programs.

#heading(level: 2)[
  Types as propositions
]

There is a second reading of what a type is. Under this reading, a type is a proposition (a logical claim) and a term of that type is a proof of that claim. The function arrow `A -> B` is the proposition "if `A` then `B`," and a lambda term inhabiting it is a proof of that implication.

This is not simply an analogy, it is actually an isomorphism between the world of types and the world of propositions called the *Curry-Howard correspondence*.

#align(center)[
  #table(
    columns: (auto, auto),
    align: (left, left),
    table.header([*Type*], [*Logic*]),
    [`A -> B`], [`A` implies `B`],
    [`A × B` (pair)], [`A` and `B`],
    [`A + B` (sum)], [`A` or `B`],
    [`Unit`], [truth],
    [`Void` (empty type)], [falsehood],
  )
]

To prove `A implies B`, you produce a procedure that, given any proof of `A`, returns a proof of `B`. This is exactly what a function of type `A -> B` does! In the bigger picture, to create a program is to construct a proof of the proposition that the program claims to satisfy.

This collapses two questions that imperative programmers have always treated as separate. "Does the program compile?" and "is the claim it makes true?" become one question and the compiler becomes a proof checker.

#heading(level: 2)[
  Why does this matter?
]

If Curry-Howard is so fundamental, why do programmers get through a career without hearing about it?

First, the types in mainstream languages are too weak to express interesting propositions. It is a funny situation because first-order logic's evolution was "we can prove things about sets, but we can't talk about functions, so let's add functions and get higher-order logic" and then programming languages' impasse was "we can talk about functions, but we can't talk about properties of values". The Java type system is not even first-order logic, it is a very limited fragment of it. It is not surprising that the propositions it can express are not interesting enough to make the Curry-Howard reading compelling.

Second, the terms in mainstream languages are not proofs because they lack totality. A C++ method of type `bool` can return `true`, return `false`, throw an exception, loop forever or halt. The Curry-Howard correspondence assumes the language is total (every well-typed term eventually produces a value of its claimed type). For instance, I shouldn't be able to do `fn proof_of_everything<T>() -> T = proof_of_everything()`, which is a proof of any proposition `T`, including falsehood!

The languages we will care about (Lean, Agda, Idris, Rocq and ultimately Lotus) are total by design or by discipline and their type systems are also strong enough to state interesting claims.

#heading(level: 2)[
  Types that depend on values
]

What separates "interesting" propositions from "uninteresting" ones we can already prove with current languages? If propositions are types according to Curry-Howard, it means that there are some types that those languages can't express.

The expressiveness of a type system for the $lambda$-calculus can be best visualized in the $lambda$-cube:

#align(center)[
  #image("assets/lambda_cube.svg", alt: "λ-cube")
]

Each vertex of the cube represents a different type system and each axis represents a different kind of dependency between types and terms that the type system allows. As we go further in each axis, we gain the ability to express more complex types.

The y-axis (up) represents terms that can depend on types. This is called *polymorphism* and is present in most modern languages, allowing us to write functions that can operate on any type, such as `length :: [a] -> Int`, which can compute the length of a list of any type `a`.

The z-axis (depth) represents types that can depend on types. Such types are called *type operators* and they allow us to write type-level functions, such as `List :: Type -> Type`, which takes a type `a` and returns the type of lists of `a`.

Finally, the x-axis (right) represents types that can depend on terms. These are called *dependent types* and they allow us to write types that can express properties of values. For example, we can write a type `Vec :: Type -> Nat -> Type`, which takes a type `a` and a natural number `n` and returns the type of vectors of `a` of length `n`. This allows us to express properties such as "this vector has length 3" or "this matrix is square".

The most famous type systems here are:
- *$lambda$$arrow$* (simply typed lambda calculus): only non-dependent function types, no polymorphism or type operators. Roughly like C (although C is not modelled by the lambda calculus)
- *$lambda$$2$* (System F): adds parametric polymorphism, allowing for generic functions. Roughly like Java/C\#/oCaml.
- *$lambda$$omega$*: supports higher-kinded types, allowing for type operators. This is where Scala lives.
- *$lambda$$omega$$2$* (System F$omega$): introduces universal polymorphism (you can abstract over both types and type operators). This is famously the basis of Haskell's type system.
- *$lambda$$omega$P2* (Calculus of Constructions): has fully dependent types, allowing for types that depend on terms. Rocq, Agda, Lean and Idris are all based on this.

So, there are two axes of the expressiveness of types that would allow for the richer propositions we were talking about. The z-axis allows us to talk about other propositions and operate on them. This is higher-order logic! Great, now we can talk about sets of sets, properties of properties, etc.

The x-axis allows us to talk about properties of values. This is what allows us to express the claim "this vector has length 3" as a type. Here is where the fun stuff happens, because we can now express properties of our programs as types and get the compiler to check them for us. For example, we can write a function `head :: Vec a (n + 1) -> a`, which takes a non-empty vector and returns its head. The type of this function guarantees that it will never be called on an empty vector, thus preventing a common source of runtime errors.

We can also prove a complex theorem about a function we wrote `encrypt :: String -> String`, for example, and say that it never produces the same output for different inputs. In Lean, this can be expressed like:

```lean4
theorem encrypt_injective : ∀ (s1 s2 : String), encrypt s1 = encrypt s2 → s1 = s2 := ...
```

Which reads roughly as "for all strings `s1` and `s2`, if `encrypt s1` is equal to `encrypt s2`, then `s1` is equal to `s2`". This is a very strong claim about the behavior of our function and if we can prove it, we can be confident that our encryption function is secure against certain types of attacks.

And `∀ (s1 s2 : String), encrypt s1 = encrypt s2 → s1 = s2` is a type. We could create a term of that type, which would be a proof of that claim. If the claim is false, then there is no term of that type and the compiler will reject our program.

The compiler becomes a proof checker and we can get very strong guarantees about our code. If we are to ever mess up in the implementation of `encrypt`, we won't be able to produce a proof of `encrypt_injective` and the compiler will tell us that our program is not correct. Instead of relying on testing (which is limited and can never provide full confidence) or code reviews (which are subjective and can miss important issues), we can get a mathematical proof that our program satisfies certain properties by reasoning about it in a purely functional way. The extent we can trust these proofs is the extent we can trust the compiler's type checker. Lean, Agda and Idris have very trustworthy minimal kernels that have been scrutinized by the community for years, so we can be very confident in the correctness of our proofs in such languages, meaning that if we can construct a proof of a claim about our program, we can be very confident that the claim is true.

#heading(level: 1)[
  Logic
]

Now we need to pick a logic to reason about our programs. The choice of logic determines what kinds of properties we can express and prove about our programs, as well as the tools and techniques we can use to do so. Ultimately, it means choosing the type system.

There are many logics we can use to reason about our programs. There are a few metrics:

1. *Expressiveness*: how much can we say in this logic? Can we talk about sets of sets, properties of properties, etc?
2. *Consistency*: are there any contradictions in this logic? Can we prove falsehood?
3. *Decidability*: can we algorithmically determine whether a given statement is provable in this logic?

For dependently typed programming languages, we want a logic that is expressive enough to talk about properties of our programs, consistent so that we don't end up proving falsehoods and ideally decidable so that we can have an algorithmic way to check proofs. However, there are trade-offs between these properties and we will see how different logics navigate these trade-offs.

#heading(level: 2)[
  Families of logics
]

We can divide logics into two broad categories: classical and intuitionistic. Both of them contain propositional logic, first-order logic and higher-order logic, but they differ in philosophy.

Classical/standard logic is the family of logics that follow the standard principles:

1. *Law of excluded middle:* every proposition is either true or false
2. *Law of non-contradiction:* no proposition is both true and false
3. *Bivalence:* only two truth values (true/false)
4. *Monotonicity of Entailment:* Adding more premises never invalidates existing conclusions.
5. *Idempotency of Entailment:* Repeating a premise does not change what can be inferred.
6. *Commutativity of Conjunction:* The order of “and” statements doesn’t affect truth (`A ∧ B ≡ B ∧ A`).
7. *De Morgan Duality:* Every logical operator has a dual via negation (`¬(A ∧ B) ≡ ¬A ∨ ¬B`, `¬(A ∨ B) ≡ ¬A ∧ ¬B`).

The philosophy here is that a statement is true or false independently of whether we can prove it.

Intuitionistic/constructive logic, on the other hand, rejects the law of excluded middle and bivalence (as well as the converse of the De Morgan duality). Here, a statement is only true if we have a proof of it and false if we have a proof of its negation. If we don't have a proof of either, then the statement is neither true nor false. This means that there are some statements that are not decidable, meaning that we can't determine whether they are true or false.

So for example, the excluded middle of a proposition `P` is `P ∨ ¬P`, which says that either we have a proof of `P` or we have a proof of `¬P`, is not a tautology because we might not have a proof of either `P` or `¬P`.

The constructive reading of logic is what we want for programming. When we say "there exists a function that satisfies this property", we want to be able to construct such a function, not just know that it exists in some abstract sense. The constructive reading of logic gives us a way to do that, it allows us to extract programs from proofs and to reason about our programs in a way that is directly connected to their implementation.

A word you will hear often in the constructivism is *witness*. A witness is a term that serves as evidence for the truth of a proposition. For example, if we have a proposition that says "there exists an integer `n` such that `n > 5`", then a witness for this proposition would be 6, for example. In the context of programming, a witness can be thought of as a concrete example or instance that demonstrates the truth of a claim.

Intuitionistic logic cares a lot about witnesses because they are the constructive evidence for the truth of a proposition. Although classical logic also accepts witnesses as proofs, it doesn't require them. Witnesses are not limited to values but also to functions. For example, if we have a proposition that says "for all integers `n`, there exists an integer `m` such that `m > n`", then a witness for this proposition would be a function that takes an integer `n` and returns an integer `m` such that `m > n`, for example, the function `f(n) = n + 1`. In Curry-Howard, witnesses are simply *values*, which make the isomorphism complete:

1. Types are propositions
2. Terms are proofs (witnesses)

Functions are merely named terms in denotational semantics, so they are also proofs. The function `f(n) = n + 1` is a witness of the proposition "for all integers `n`, there exists an integer `m` such that `m > n`" because it provides a way to construct such an integer `m` for any given integer `n`.

So we have to go with intuitionistic logic for our programming language, classical logic's philosophy of truth doesn't align with the constructive nature of programming. However, we can still use classical logic as a tool for reasoning about our programs, as long as we are careful to only use it in ways that are compatible with the constructive reading of logic. For example, we can use classical logic to prove theorems about our programs, but we should not use it to make claims about the existence of certain functions or properties without providing a constructive witness for those claims.

#heading(level: 2)[
  Propositional logic
]

Propositional logic is the simplest form of classical logic, it deals only with whole propositions (statements that are true or false) and the connectives between them without variables, quantifiers or internal structure.

The propositions are atomic, you can't look inside them or talk about their properties, they are just black boxes that can be true or false.

Not all connectives are independent, you can express every possible truth function using just:

- ${ not, and}$
- ${ not, or}$
- ${ not, =>}$
- or even a single connective: NAND ($arrow.t$) or NOR ($arrow.b$) alone

This is called *functional completeness*, which means that a set of connectives is sufficient to express all possible truth functions. Every digital circuit can be built from NAND alone, which is just propositional logic. In fact, you can even make a computer out of NAND gates! @NAND. This is basically how our CPUs already work, they are just a big network of NAND gates.

Since propositional logic is so simple, it is decidable. We can algorithmically determine whether a given statement is provable in propositional logic by using truth tables or other methods.

But propositional logic is not very expressive since it can't talk about the internal structure of propositions or about properties of values.

#heading(level: 2)[
  First-order logic
]

First-order logic extends propositional logic by introducing quantifiers and variables that can range over a domain of discourse. This allows us to talk about the internal structure of propositions and to express properties of values.

"Order" refers to what you're allowed to quantify over. In first-order logic, you can only quantify over individuals (elements of your domain), never over predicates or functions themselves.

So you can say $forall x, P(x)$ ("for all elements x, predicate P holds") but you can't say $forall P \, P(x)$ ("for all predicates P, P holds of x").

Therefore, first-order logic technically doesn't have induction, but we can encode it as an axiom schema. An axiom schema is a template that generates infinitely many axioms, one for each formula you plug into it. Say you want to define the natural numbers. You have zero, successor and you want induction. Induction says: if some property holds for 0 and whenever it holds for n it also holds for n+1, then it holds for all natural numbers.

What you can do is write induction for one specific property at a time. Say you want to prove that every natural number is either even or odd, you write:

$"EvenOrOdd"(0) and forall n, ("EvenOrOdd"(n) => "EvenOrOdd"(n+1)) => forall n, "EvenOrOdd"(n)$

That's a perfectly valid first-order sentence. It's just the induction principle with $P$ replaced by one concrete formula.

If you are paying attention, this should've reminded you of polymorphism and monomorphization. In programming languages, we have a similar situation where we want to write a function that works for all types, such as `length :: [a] -> Int`, but we can't directly express that in a language without polymorphism. Instead, we can write a monomorphized version of the function for each type we want to support, such as `lengthInt :: [Int] -> Int`, `lengthString :: [String] -> Int`, etc. This is similar to how we can write an axiom schema for induction in first-order logic.

First-order logic is complete, sound, semi-decidable and compact:
1. *Completeness* means that if a statement is true in all models of a theory, then it is provable from the axioms of that theory. In other words, if something is semantically valid, then it is syntactically provable.
2. *Soundness* means that if a statement is provable from the axioms of a theory, then it is true in all models of that theory. In other words, if something is syntactically provable, then it is semantically valid.
3. *Semi-decidable* means that there is an algorithm that can determine whether a given statement is provable from the axioms of a theory, but there is no algorithm that can determine whether a given statement is not provable.
4. *Compactness* means that if every finite subset of a theory has a model, then the whole theory does. This is a very powerful property that allows us to build infinite structures by ensuring that all of their finite substructures are consistent.

First-order logic is a very well-behaved logic that has many nice properties, but it is not very expressive.

#heading(level: 2)[
  Higher-order logic
]

In contrast, higher-order logic lets you quantify over predicates, functions and relations. You can say things like "for all properties P, if P holds for 0 and P is preserved by successor, then P holds for all natural numbers".

That looks like $forall P, P(0) and forall n, (P(n) => P(n+1)) => forall n, P(n)$.

This is the fundamental trade-off. Higher-order logic is dramatically more expressive:

- It can categorically define the natural numbers by Peano's axioms (up to isomorphism)
- It can categorically define the reals as Dedekind cuts (up to isomorphism)
- It can express the induction principle as a single axiom
- It can define finiteness, well-ordering and continuity directly

But you pay for this. Gödel's completeness theorem fails (there are higher-order truths with no formal proof) and you also lose compactness. In a sense, higher-order logic is so expressive that no proof system can fully capture it. This is why first-order logic became the standard foundation for mathematics, because it's the sweet spot where you still have a complete proof system.

However, that limitation is not the end of the story. There are two main semantics for higher-order logic:

1. Standard semantics: predicate variables range over all subsets/relations, which gives you full expressiveness but breaks completeness.
2. Henkin semantics: predicate variables range over a specified (possibly restricted) collection of subsets, which recovers completeness but is essentially equivalent to a many-sorted first-order theory.

When people say "higher-order logic is incomplete" they mean standard semantics. Most proof assistants use something closer to Henkin semantics in practice.

In fact, higher-order logic is the standard in type theory. System F (polymorphic lambda calculus) is essentially second-order logic under the Curry-Howard correspondence:

- A polymorphic type like `forall a. a -> a` is a second-order universal quantification, it quantifies over types (which play the role of predicates)
- Type constructors like `List : Type -> Type` are higher-order (they're functions on types)

So higher-order logic's expressiveness is what we're looking for to prove interesting properties about our polymorphic and higher-order programs.

#heading(level: 2)[
  Proof Systems
]

Up to this point we've been talking about "proving" things. But what is a proof, concretely? How do we construct one? The same way a programmer needs a formal grammar for constructing programs, a logician needs a formal proof system for constructing proofs.

There are three main families of proof systems which we will cover, but *natural deduction* is the one we will spend the most time with.

#heading(level: 3)[
  Hilbert-style systems
]

Hilbert-style systems came first descending from Russell-Whitehead's _Principia Mathematica_, which tried, but ultimately failed, to derive all of mathematics from logic (and has the infamous proof of $1 + 1 = 2$ that takes hundreds of pages).

They take one extreme of the design space, which is more minimalistic with a tiny number of inference rules (which dictate how you can derive new truths from old ones) and a large number of axiom schemas.

Inference rules are written with a horizontal bar separating the premises (what you need to already have, above) from the conclusion (what you can derive, below). The $Gamma$ on the left of the turnstile $tack.r$ represents the context of assumptions that you have available to you when applying the rule. For example, modus ponens looks like:

$ frac(Gamma tack.r A => B quad Gamma tack.r A, Gamma tack.r B) quad ("MP") $

It reads as: if from $Gamma$ you can derive $A => B$ and from $Gamma$ you can also derive $A$, then from $Gamma$ you can derive $B$.

Hilbert systems are equivalent to natural deduction and sequent calculus in the sense that they prove the same propositions, but the process of proving is miserable. What rescues Hilbert spaces in practice is the deduction theorem, which states that if $Gamma, A tack.r B$ (from $Gamma$ together with $A$ you can derive $B$) is provable, then $Gamma tack.r A => B$ ($Gamma$ you can derive $A => B$) is provable.

Their minimal rule set makes them excellent for metalogical work. When you're trying to prove things about a logic, completeness, consistency, conservativity, soundness over some semantics, fewer rules means fewer cases to handle in the metaproof (think about pattern matching for example, it's easier to write a function with 2 cases than with 20 cases). They can be thought of as the assembly language of formal logic as no one writes in them by choice, but they're preferred when reasoning about the machine itself.

#heading(level: 3)[
  Natural deduction
]

Natural deduction takes the opposite tradeoff from Hilbert systems. Instead of many axioms and few inference rules, it has no axioms but does provide a generous collection of inference rules. The rules come in pairs, so for each connective an *introduction* rule that says how to build a proof of a proposition involving that connective and an *elimination* rule that says how to use such a proof. This is the proof system whose shape maps directly onto type theory and it's the one we'll spend the most time with.

*Judgments.* The natural deduction judgment looks like:

$ Gamma tack.r A $

reads as "from the assumptions in $Gamma$, we can derive $A$".

$Gamma$ is a context (a list of propositions assumed to hold) and the turnstile $tack.r$ separates assumptions from the conclusion. Every proof rule we're about to see will be a statement about judgments, not about propositions in isolation.

*Conjunction.* The introduction rule for $and$ says: from a proof of $A$ and a proof of $B$, you can form a proof of $A and B$. The two elimination rules say: from a proof of $A and B$, you can pull out a proof of either side. In rule form:

$
  frac(Gamma tack.r A quad Gamma tack.r B, Gamma tack.r A and B) quad (and "-I") quad quad frac(Gamma tack.r A and B, Gamma tack.r A) quad (and "-E"_1) quad quad frac(Gamma tack.r A and B, Gamma tack.r B) quad (and "-E"_2)
$

Programmers can already smell what's coming! $and$-I is the constructor for a pair and $and$-E#sub[1] and $and$-E#sub[2] are `fst` and `snd`. But hold that thought.

*Implication.* The introduction rule says: to prove $A => B$, temporarily assume $A$, prove $B$ under that assumption, then "discharge" the assumption when you form the implication (the assumption joins the context above the bar in the premise, then disappears below the bar in the conclusion). The elimination rule is just modus ponens again (given $A => B$ and $A$, conclude $B$). In rule form:

$
  frac(Gamma comma A tack.r B, Gamma tack.r A => B) quad (=> "-I") quad quad frac(Gamma tack.r A => B quad Gamma tack.r A, Gamma tack.r B) quad (=> "-E")
$

The shape of $=>$-I is exactly the typing rule for a function. To type-check $lambda x. e : A arrow.r B$ ("the function that takes an argument $x$ and returns $e$ has type $A => B$"), we assume $x : A$ and try to derive $B$ from that assumption. If we can, then we can conclude that the whole function has type $A => B$.

And $=>$-E is just function application. If we have a term of type $A => B$ and a term of type $A$, we can apply the function to the argument and get a term of type $B$.

*Disjunction.* To prove a disjunction, pick a side and prove it (those are the two introduction rules). To use a disjunction, handle both cases (like pattern matching `match` an ADT/enum). In rule form:

$
  frac(Gamma tack.r A, Gamma tack.r A or B) quad (or "-I"_1) quad quad frac(Gamma tack.r B, Gamma tack.r A or B) quad (or "-I"_2)
$

$ frac(Gamma tack.r A or B quad Gamma comma A tack.r C quad Gamma comma B tack.r C, Gamma tack.r C) quad (or "-E") $

Note how $or$-E threads the goal $C$ through both branches. That is because, to use a disjunction, you must show that no matter which side it came from, you reach the same conclusion. Suppose we have a proof of $A or B$ and we want to derive $C$. We don't know whether the proof of $A or B$ came from $A$ or from $B$, so we have to consider both cases. If it came from $A$, then we assume $A$ and try to derive $C$. If it came from $B$, then we assume $B$ and try to derive $C$. If we can derive $C$ in both cases, then we can conclude that $C$ follows from $A or B$.

*Falsehood and truth.* Falsehood has only an elimination rule and no introduction (since you can't construct falsehood). The elimination rule says that if you've somehow derived falsehood from your assumptions, you can derive anything else. This is called _ex falso quodlibet_, "from falsehood, whatever you like":

A famous legend is that Bertrand Russell, while teaching the _ex falso quodlibet_ to his students, was asked to prove he is the Pope given `1 = 0`, to which he replied "add 1 to both sides and you get `2 = 1`. The set of me and the Pope has 2 members, and `2 = 1`, so it has only 1 member, so I am the Pope".

$ frac(Gamma tack.r bot, Gamma tack.r C) quad (bot "-E") $

In a consistent system, you can never produce a proof of $bot$ from no assumptions. Truth $top$ is the opposite since an introduction rule ($top$ always holds) and no elimination rule (you can't extract any information from a proof of $top$).

*Quantifiers.* $forall x. P(x)$ is introduced by proving $P(x)$ for a generic $x$ that doesn't appear free in $Gamma$ and eliminated by instantiating at a specific term $t$ to get $P(t)$. $exists x. P(x)$ is introduced by providing a witness $t$ together with a proof of $P(t)$ and eliminated by case-analyzing on that pair.

Let's try to prove $A and B => B and A$ to see how the rules compose. We're trying to derive $tack.r A and B => B and A$ from the empty context. The only rule that produces an implication in the conclusion is $=>$-I, so we work backwards and assume $A and B$ and try to derive $B and A$ under that assumption.

Now our goal is $A and B tack.r B and A$. The only rule that produces a conjunction in the conclusion is $and$-I, which needs two subgoals: $A and B tack.r B$ and $A and B tack.r A$. For each, we apply the relevant elimination rule to the assumption $A and B$: $and$-E#sub[2] gives us $B$, $and$-E#sub[1] gives us $A$. Both subgoals collapse to direct uses of the assumption.

Read top-down, the full proof is: from the assumption $A and B$, pull out $A$ via $and$-E#sub[1] and $B$ via $and$-E#sub[2]; combine them in reverse order with $and$-I to get $B and A$; discharge the assumption with $=>$-I to obtain $A and B => B and A$.

If you look at the intro and elim rules for a connective side by side, you'll notice that the introduction rule condenses information in and the elimination rule pulls the same information back out. For instance, if you intro a pair and then immediately project, you get back what you put in. That is expected, the elimination rule should be neither stronger nor weaker than what the introduction rule justifies, since if, for example, you eliminate more than you introduced, the system is unsound (you'd be extracting information that was never there) and if you eliminate less, it's incomplete (you couldn't fully use what you constructed). This concept is referred to as *harmony*.

#heading(level: 3)[
  Sequent calculus
]

Sequent calculus is primarily used as a tool for proving consistency. The judgments are more symmetric:

$ Gamma tack.r Delta $

where both $Gamma$ and $Delta$ are lists of formulas. The intended reading is "if all the formulas in $Gamma$ hold, then at least one of the formulas in $Delta$ holds". So the left is a conjunction of assumptions and the right is a disjunction of conclusions. The turnstile $tack.r$ still separates assumptions from conclusions, but now we have multiple conclusions instead of just one.

Classical sequent calculus (LK) lets the right-hand side $Delta$ hold any number of formulas, while intuitionistic sequent calculus (LJ) restricts $Delta$ to at most one. It's surprising that this single restriction is the whole difference between classical and intuitionistic logic given that the usual story about intuitionism is about constructing objects and giving direct evidence. In classical logic, a proof of $A$ can just be a demonstration that $not A$ leads to a contradiction. However, intuitionistically, a proof of $A or B$ has to say which disjunct holds and a proof of $exists x. P(x)$ has to produce a concrete witness $t$ and a proof of $P(t)$.

A multiple-conclusion sequent $Gamma tack.r B_1, dots, B_n$ is read disjunctively ($B_1 or dots.c or B_n$) and you can prove it without ever deciding which $B_i$ holds. To prove $exists x. P(x)$ in LK, you keep the $exists x. P(x), forall x. not P(x)$ open and never commit to a particular $x$. In LJ, since only one formula is allowed on the right, the applicable existential rule only applies to a single $P(t)$, so you must choose a concrete $t$ and prove $P(t)$.

Each connective gets left/right rules instead of intro/elim. The right rules look almost identical to natural deduction's introductions, the right rule for $and$ is just $and$-I:

$ frac(Gamma tack.r A quad Gamma tack.r B, Gamma tack.r A and B) quad (and "R") $

The left rules describe how to use an assumption containing the connective:

$ frac(Gamma comma A comma B tack.r C, Gamma comma A and B tack.r C) quad (and "L") $

Implication is symmetric in the same way:

$
  frac(Gamma comma A tack.r B, Gamma tack.r A => B) quad (=> "R") quad quad frac(Gamma tack.r A quad Gamma comma B tack.r C, Gamma comma A => B tack.r C) quad (=> "L")
$

The right rule discharges an assumption to build an implication, just like $=>$-I. The left rule says how to consume an implication. In this case, if you can prove $A$ and you can prove $C$ assuming $B$, then you can prove $C$ from $A => B$. If you're paying attention, this resembles modus ponens, but backwards. Instead of "given $A => B$ and $A$, get $B$ going forward", it's "if you have $A => B$ lying around, use it to reduce a goal $C$ to two subgoals".

There's a special rule called cut:

$ frac(Gamma tack.r A quad Gamma comma A tack.r B, Gamma tack.r B) quad ("Cut") $

Cut is the formalization of using a lemma, if you can prove $A$ standalone and you can prove $B$ assuming $A$, then you can prove $B$ directly.

Every derivation using the cut rule can be transformed into one that doesn't. This is sequent calculus's way of simplifying proofs, counterpart to proof normalization in natural deduction which removes detours through intro-then-elim pairs and $lambda$-calculus's $Beta$-reduction.

There are three main rules: weakening (add an unused assumption), contraction (collapse two copies of the same assumption into one) and exchange (reorder assumptions).

When we get rid of weakening, we get linear logic, where assumptions are resources that must be used exactly once. When we get rid of contraction, we get affine logic, where assumptions are resources that must be used at most once. When we get rid of exchange, we get ordered logic, where the order of assumptions matters.

That is beautiful! By tweaking the structural rules of the sequent calculus, we can get different logics that have different philosophies and applications.

#heading(level: 2)[
  The BHK interpretation
]

The three proof systems we just saw all tell us how to construct proofs, but none of them tell us what a proof is. In classical logic, a proof is akin to a certificate that the proposition is true and doesn't carry much further meaning. Intuitionistic logic, under the lens of the BHK interpretation, is different:

- A proof of $A and B$ is a *pair* of a proof of $A$ and a proof of $B$.
- A proof of $A or B$ is a *tagged* proof: either a proof of $A$ labeled "left" or a proof of $B$ labeled "right".
- A proof of $A => B$ is a *construction* (function, method, procedure) that, given any proof of $A$, produces a proof of $B$.
- A proof of $bot$ is impossible, there is no such proof.
- A proof of $top$ is trivial.
- A proof of $forall x. P(x)$ is a *construction* that, given any element $t$ of the domain, produces a proof of $P(t)$.
- A proof of $exists x. P(x)$ is a *pair* of a specific witness $t$ and a proof of $P(t)$.

We started this chapter trying to formalize what a proof is, now we can say that proofs are data structures like pairs, tagged unions, functions, witnesses, etc.

#heading(level: 3)[
  Curry-Howard
]

We saw the Curry-Howard correspondence as a small table back in the section on types as propositions. Now that we are leveled up, we can write a more complete table:

#align(center)[
  #table(
    columns: (auto, auto),
    align: (left, left),
    table.header([*Natural deduction*], [*Simply-typed lambda calculus*]),
    [Proposition $A$], [Type $A$],
    [Proof of $A$], [Term of type $A$],
    [Assumption $Gamma tack.r A$], [Typing context $Gamma tack.r e : A$],
    [$A and B$], [Product type $A times B$],
    [$and$-I], [Pair constructor $(a, b)$],
    [$and$-E#sub[1], $and$-E#sub[2]], [`fst`, `snd`],
    [$A or B$], [Sum type $A + B$],
    [$or$-I#sub[1], $or$-I#sub[2]], [Injections `inl`, `inr`],
    [$or$-E], [Pattern match],
    [$A => B$], [Function type $A arrow.r B$],
    [$=>$-I], [Lambda abstraction $lambda x. e$],
    [$=>$-E], [Application $f thin a$],
    [$bot$], [Empty type `Void`],
    [$bot$-E], [`absurd : Void -> A`],
    [$top$], [Unit type],
    [$forall x. P(x)$], [Dependent function type $(x : T) arrow.r P thin x$],
    [$exists x. P(x)$], [Dependent pair type $(x : T) times P thin x$],
  )
]

The natural deduction rule $=>$-I

$ frac(Gamma comma A tack.r B, Gamma tack.r A => B) $

with terms decorated, becomes

$ frac(Gamma comma x : A tack.r e : B, Gamma tack.r (lambda x. e) : A arrow.r B) $

This generalizes, see below:

#align(center)[
  #image("assets/hm.svg")
]

#heading(level: 3)[
  Proof normalization is $Beta$-reduction
]

A derivation that intros and then immediately elims can be normalized because of harmony. For conjunction, a derivation like:

$ and "-E"_1 thin (and "-I" thin (a, b)) $

normalizes to just $a$, the pair was constructed and then projected. Under Curry-Howard, that proof normalization step is:

$ "fst"(a, b) quad arrow.r.long quad a $

For implication, intro-then-elim looks like

$ => "-E" thin (=> "-I" thin (lambda x. e), thin a) $

which normalizes to $e$ with $a$ substituted for $x$. As a program:

$ (lambda x. e) thin a quad arrow.r.long quad e[a slash x] $

This is $beta$-reduction!

Proof normalization in natural deduction is $beta$-reduction in the simply-typed lambda calculus, therefore computing a program and simplifying a proof are the same operation. When you run a program, you're normalizing a proof. When you simplify a proof by eliminating redundant intro/elim pairs, you're executing a program. This is also exactly cut elimination in sequent calculus!

#heading(level: 2)[
  Kripke semantics
]

This is where Lotus starts to diverge from the usual presentation of intuitionistic logic. Stereotypically, Kripke, as a philosopher, wondered about the meaning of an intuitionistic proposition. In classical logic, the meaning of a proposition is its truth value, but intuitionistic logic doesn't admit bivalence (binary true/false), so that can't be it.

Kripke's insight was to give a semantics in terms of possible worlds that evolve over time.

*Possible worlds.* A Kripke model for intuitionistic logic consists of:

1. A set $W$ of worlds (states of knowledge, moments in time, stages of investigation).
2. A relation $lt.eq$ on $W$, where $w lt.eq w'$ means "world $w'$ is reachable from $w$" or "what's true at $w$ is still true at $w'$".
3. For each world $w$, a set of atomic propositions that hold at $w$.

The relation $lt.eq$ captures the intuition that knowledge accumulates, as we can only learn new facts and never forget old ones. A proposition that holds at a world $w$ must also hold at all future worlds $w' gt.eq w$.

*The forcing relation.* Forcing is what that tells us whether a proposition holds at a world. We write $w tack.r.double A$ and read it "world $w$ forces $A$" or "$A$ holds at $w$". For atomic $A$, forcing is given by the model and for complex propositions, we define it recursively:

- $w tack.r.double A and B$ iff $w tack.r.double A$ and $w tack.r.double B$.
- $w tack.r.double A or B$ iff $w tack.r.double A$ or $w tack.r.double B$.
- $w tack.r.double A => B$ iff for _every_ $w' gt.eq w$, if $w' tack.r.double A$ then $w' tack.r.double B$.
- $w tack.r.double not A$ iff for every $w' gt.eq w$, $w'$ does not force $A$.

This is what gives implication the right constructive content, since to know $A => B$ at world $w$, you need to know that no matter how your knowledge develops, if you ever come to know $A$, you'll come to know $B$.

*Law of Excluded Middle.* We can use this as another demonstration of why $A or not A$ is not true in intuitionistic logic. Consider a model with two worlds, $w_0 lt.eq w_1$ where a proposition $P$ doesn't hold at $w_0$ but does hold at $w_1$:

- At $w_0$: $P$ doesn't hold, so $w_0$ does not force $P$.
- At $w_0$: is $not P$ forced? That requires that for every future world $w' gt.eq w_0$, $w'$ doesn't force $P$. But $w_1$ does force $P$, so no, $not P$ is not forced at $w_0$ either.
- Therefore $w_0$ does not force $P or not P$.

$P or not P$ fails at a world that hasn't yet decided whether $P$ holds. Intuitionism can be thought of as a logic of evolving knowledge and LEM fails because there are times we need to be humble and say "I don't know yet"!

A classical model is just a Kripke model with one world, so classical logic is intuitionistic logic at a single moment!

We will come back to Kripke semantics in the chapter on modal logic.

#bibliography("works.bib", style: "nature")
