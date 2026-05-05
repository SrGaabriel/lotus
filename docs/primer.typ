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

This document serves as a primer for the concepts and motivations behind the development of Lotus, a dependently typed programming language. we will explore everything from the basics of dependent types, obscure math properties, etc.

#heading(level: 1, numbering: "1")[
  What Programs Are
]

#heading(level: 2, numbering: "1.1")[
  Models of Computation
]

There exist two main models of computation: the Turing machine and the lambda calculus. The Turing machine is a theoretical model that consists of a tape divided into cells, a head that can read and write symbols on the tape and a state register that holds the current state of the machine. Everyone is familiar with the Turing machine, as it is the most widely used model of computation and is often used to define what it means for a function to be computable.

The lambda calculus, on the other hand, is a bit less known. It is a formal system for expressing computation that consists of exactly three constructs: variables, function definitions and function applications. It doesn't have a notion of state, memory, time, strings, numbers, etc. It is a purely functional model of computation where everything is a function and computation is done by applying functions to arguments.

Both are equivalent in terms of computational power, they are both Turing-Complete, which means that they can compute any function that is computable and can encode each other. You may find this surprising and ask "how can I write a text editor without strings?". You are already doing it, your CPU doesn't have a notion of strings, it operates only on bits. But we made it interpret certain combinations of bits as strings, numbers, etc. You can do the same with the lambda calculus.

The lambda calculus is a more abstract model of computation that focuses on the notion of functions and their application, while the Turing machine is a more concrete model that focuses on the notion of state and memory. Both models are useful for different purposes and have their own advantages and disadvantages.

The Turing machine is considered more efficient because our computers are designed to operate on a von Neumann architecture, which is based on the Turing machine model. The lambda calculus, on the other hand, is more elegant and easier to mathematically reason about, which is why it is often used in the study of programming languages and type systems. That is because 99% of the field of logic is immutable and stateless, which is a perfect fit for the lambda calculus. The Turing machine is better suited for modeling stateful computations, while the lambda calculus is better suited for modeling pure computations.

#heading(level: 2, numbering: "1.2")[
  Values and Transformations
]
