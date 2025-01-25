// math.rs

#![allow(clippy::doc_markdown)]

//! Math module.
//!
//! There's not much we can do right now except test inline math such as $\sum_{i=1}^{n} i =
//! \frac{n(n+1)}{2}$.
//!
//! Testing $\KaTeX$, not to be confused with $\LaTeX$. Here are some examples:
//!
//! $$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}.$$
//!
//! $$
//! \frac{1}{\Bigl(\sqrt{\phi \sqrt{5}}-\phi\Bigr) e^{\frac25 \pi}} = 1+\frac{e^{-2\pi}} {1+\frac{e^{-4\pi}}
//! {1+\frac{e^{-6\pi}} {1+\frac{e^{-8\pi}} {1+\cdots} } } }. $$
//!
//! $$
//! \left( \sum_{k=1}^n a_k b_k \right)^2 \leq \left( \sum_{k=1}^n a_k^2 \right) \left( \sum_{k=1}^n b_k^2
//! \right). $$
//!
//! $$
//! {1 + \frac{q^2}{(1-q)}+\frac{q^6}{(1-q)(1-q^2)}+\cdots} =
//! \prod_{j=0}^{\infty}\frac{1}{(1-q^{5j+2})(1-q^{5j+3})}, \quad\quad \text{for }\lvert q\rvert<1. $$

/// Some math function.
pub fn some_math_function() {}

// EOF
