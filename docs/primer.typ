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

This document serves as a primer for the concepts and motivations behind the development of Lotus, a dependently typed programming language. We will explore everything from the basics of dependent types, obscure math properties, etc.

#heading(level: 1)[
  What Programs Are
]

#heading(level: 2)[
  Models of Computation
]

There exist two models of computation we'll focus on: the Turing machine and the lambda calculus. The Turing machine is a theoretical model that consists of a tape divided into cells, a head that can read and write symbols on the tape and a state register that holds the current state of the machine. Everyone is familiar with the Turing machine, as it is the most widely used model of computation and is often used to define what it means for a function to be computable.

The lambda calculus, on the other hand, is a bit less known. It is a formal system for expressing computation that consists of exactly three constructs: variables, function definitions and function applications. It doesn't have a notion of state, memory, time, strings, numbers, etc. It is a purely functional model of computation where everything is a function and computation is done by applying functions to arguments.

Both are equivalent in terms of computational power, they are both Turing-Complete, which means that they can compute any function that is computable and can encode each other. You may find this surprising and ask "how can I write a text editor without strings?". You are already doing it, your CPU doesn't have a notion of strings, it operates only on bits. But we made it interpret certain combinations of bits as strings, numbers, etc. You can do the same with the lambda calculus.

The lambda calculus is a more abstract model of computation that focuses on the notion of functions and their application, while the Turing machine is a more concrete model that focuses on the notion of state and memory. Both models are useful for different purposes and have their own advantages and disadvantages.

The imperative model is considered more efficient because our computers are designed to operate on a von Neumann architecture, which is based on the Turing machine. The lambda calculus, on the other hand, is more elegant and easier to mathematically reason about, which is why it is often used in the study of programming languages and type systems. That is because 99% of the field of logic is immutable and stateless, which is a perfect fit for the lambda calculus. The Turing machine is better suited for modeling stateful computations, while the lambda calculus is better suited for modeling pure computations.

#heading(level: 2)[
  What is a Program?
]

In the Java/C/Python lineage, a program is fundamentally a sequence of instructions that mutate state over time. This is called the imperative paradigm, where programs are scripts that tell the machine what to do at each tick. A method is a procedure that takes inputs, possibly reads or writes shared state, possibly performs I/O, possibly throws, possibly exits or eventually returns.

Types in this world are mostly a tagging system for memory layouts and a sanity check on method calls, they simply describe what kind of bits sit at an address.

The meaning of a program under this philosophy is "what it does when you run it". This is called *operational semantics*. When reasoning about programs under this view, we are basically running a mental simulation of the machine executing the program. This is very intuitive and is how most programmers think about their code, however, it has some drawbacks. It is hard to reason about the behavior of a program without actually running it, especially when the program has side effects or is non-deterministic. It is also hard to reason about the correctness of a program, because you have to consider all possible states and inputs that the program can encounter.

In the lambda calculus lineage, programs are just expressions that denote values. For example, given `factorial :: Int -> Int`, the expression `factorial 5` doesn't denote the process of calculating the factorial of 5, it denotes the value 120 in the same way that `2 + 2` denotes the value 4.

To execute a program in this world, we need to evaluate the expression until we get a value. This descends from the lambda calculus where computation is just the process of reducing an expression to its normal form.

This is called *denotational semantics*, since we purely reason about what our expressions denote and how they compose. This model gives us *referential transparency*, meaning that we can replace an expression with its value without changing the meaning of the program. We call this paradigm *functional programming*, since we focus on the composition of pure functions that take inputs and produce outputs without side effects. This makes it easier to reason about our code, since we don't have to worry about the state of the world or the order of execution.

So how do we execute side-effectful programs in pure functional programming? The answer is *monads*, but we will get to that later.

#heading(level: 1)[
  Types
]

#heading(level: 2)[
  Types as bookkeeping
]

In the imperative paradigm, a type is a label on bits. `int` signifies that the next four bytes should be read as a two's-complement integer, `String` represents a pointer to a heap-allocated sequence of characters, etc. The type system's job is to make sure you don't read those bytes the wrong way and that you don't conflate different kinds of data.

This is a useful job as it catches a lot of bugs at compile time and types describe layout and prevent confusion. However, they do not say much about what the program means. For example, a function with type `int -> int` could be either the factorial or a function that erases your hard drive and returns 0.

The internet is full of debates on static vs dynamic typing, but they are mostly arguing about the ergonomics of the bookkeeping system, mainly whether the safety checks are worth the extra annotations and development time.

The deeper question, however, about what a type is and what it means is not really on the table. The next section will show that there is a much richer reading of what a type is and that it has profound implications for how we write and reason about programs.

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

So for example the excluded middle of a proposition `P` is `P ∨ ¬P`, which says that either we have a proof of `P` or we have a proof of `¬P`, is not a tautology here because we might not have a proof of either `P` or `¬P`.

The constructive reading of logic is exactly what we need for programming. When we say "there exists a function that satisfies this property", we want to be able to construct such a function, not just know that it exists in some abstract sense. The constructive reading of logic gives us a way to do that, it allows us to extract programs from proofs and to reason about our programs in a way that is directly connected to their implementation.

A word you will hear often in the constructivism is *witness*. A witness is a term that serves as evidence for the truth of a proposition. For example, if we have a proposition that says "there exists an integer `n` such that `n > 5`", then a witness for this proposition would be 6, for example. In the context of programming, a witness can be thought of as a concrete example or instance that demonstrates the truth of a claim. For instance, if we have a function that claims to return a sorted list, then a witness for this claim would be an actual sorted list that the function returns when given some input.

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

Therefore, first-order logic is a very well-behaved logic that has many nice properties, but it is not very expressive and compactness can be a double-edged sword since it can lead to unintended consequences, such as the existence of non-standard models of arithmetic.

#heading(level: 2)[
  Higher-order logic
]

Where First-Order Logic only lets you quantify over individuals, higher-order logic lets you quantify over predicates, functions and relations themselves. You can say things like "for all properties P, if P holds for 0 and P is preserved by successor, then P holds for all natural numbers", that's the induction principle which is second-order.

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

#bibliography("works.bib", style: "nature")
