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

The expressiveness of a type system can be best visualized in the $lambda$-cube:

#align(center)[
  #image("assets/lambda_cube.svg", alt: "λ-cube")
]

Each vertex of the cube represents a different type system for the lambda calculus and each axis represents a different kind of dependency between types and terms.

The y-axis (up) represents terms that can depend on types. This is called *polymorphism* and is present in most modern languages and allows us to write functions that can operate on any type, such as `length :: [a] -> Int`, which can compute the length of a list of any type `a`.

The z-axis (depth) represents types that can depend on types. Such types are called *type operators* and they allow us to write type-level functions, such as `List :: Type -> Type`, which takes a type `a` and returns the type of lists of `a`.

Finally, the x-axis (right) represents types that can depend on terms. These are called *dependent types* and they allow us to write types that can express properties of values. For example, we can write a type `Vec :: Type -> Nat -> Type`, which takes a type `a` and a natural number `n` and returns the type of vectors of `a` of length `n`. This allows us to express properties such as "this vector has length 3" or "this matrix is square".

The most famous type systems here are:
- *$lambda$$arrow$* (simply typed lambda calculus): only non-dependent function types, no polymorphism or type operators. Roughly like C (although C is not modelled by the lambda calculus)
- *$lambda$$2$* (System F): adds parametric polymorphism, allowing for generic functions. Roughly like Java/C\#/oCaml.
- *$lambda$$omega$*: supports higher-kinded types, allowing for type operators. This is where Scala lives.
- *$lambda$$omega$$2$* (System F$omega$): has universal polymorphism (you can abstract over both types and type operators). This is famously the basis of Haskell's type system.
- *$lambda$$omega$P2* (Calculus of Constructions): adds dependent types, allowing for types that depend on terms. Rocq, Agda, Lean and Idris are all based on this.

So, there are two axis of the expressivity of types that would allow for richer propositions. The z-axis allows us to talk about other propositions and operate on them. This is higher-order logic! Great, now we can talk about sets of sets, properties of properties, etc.

The x-axis allows us to talk about properties of values. This is what allows us to express the claim "this vector has length 3" as a type. Here is where the fun stuff happens, because we can now express properties of our programs as types and get the compiler to check them for us. For example, we can write a function `head :: Vec a (n + 1) -> a`, which takes a non-empty vector and returns its head. The type of this function guarantees that it will never be called on an empty vector, thus preventing a common source of runtime errors.

We can also prove a complex theorem about a function we wrote `encrypt :: String -> String`, for example, and say that it never produces the same output for different inputs. In Lean, this can be expressed like:

```lean4
theorem encrypt_injective : ∀ (s1 s2 : String), encrypt s1 = encrypt s2 → s1 = s2 := ...
```

Which reads roughly as "for all strings `s1` and `s2`, if `encrypt s1` is equal to `encrypt s2`, then `s1` is equal to `s2`". This is a very strong claim about the behavior of our function and if we can prove it, we can be confident that our encryption function is secure against certain types of attacks.

And `∀ (s1 s2 : String), encrypt s1 = encrypt s2 → s1 = s2` is a type. We could create a term of that type, which would be a proof of that claim. If the claim is false, then there is no term of that type and the compiler will reject our program.

The compiler becomes a proof checker and we can get very strong guarantees about our code. If we are to ever mess up in the implementation of `encrypt`, we won't be able to produce a proof of `encrypt_injective` and the compiler will tell us that our program is not correct. Instead of relying on unit tests which couldn't possibly cover all cases, we can get a mathematical proof that our program satisfies certain properties by reasoning about it in a purely functional way.

#heading(level: 1)[
  Logic
]

