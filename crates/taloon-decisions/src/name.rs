//! Byte-domain validation and Python package-name canonicalization.
#[cfg(creusot)]
use creusot_std::prelude::*;

// Numeric ASCII literals in logic avoid a Creusot 0.14 byte-literal translation bug.
#[cfg(creusot)]
#[logic(open, inline)]
pub fn alphanumeric(b: u8) -> bool {
    pearlite! { (48u8 <= b && b <= 57u8) || (65u8 <= b && b <= 90u8) || (97u8 <= b && b <= 122u8) }
}

#[cfg(creusot)]
#[logic(open, inline)]
pub fn separator(b: u8) -> bool {
    pearlite! { b == 45u8 || b == 95u8 || b == 46u8 }
}

#[cfg(creusot)]
#[logic(open, inline)]
pub fn valid(input: Seq<u8>) -> bool {
    pearlite! {
        input.len() > 0 && alphanumeric(input[0]) && alphanumeric(input[input.len()-1])
        && forall<i> 0 <= i && i < input.len() ==> alphanumeric(input[i]) || separator(input[i])
    }
}

#[cfg(creusot)]
#[logic(open, inline)]
pub fn lowercase(b: u8) -> u8 {
    pearlite! { if 65u8 <= b && b <= 90u8 { b + 32u8 } else { b } }
}

#[cfg(creusot)]
#[logic(open)]
#[requires(0 <= n && n <= input.len())]
#[variant(n)]
pub fn normalized_prefix(input: Seq<u8>, n: Int) -> Seq<u8> {
    pearlite! {
        if n == 0 { Seq::empty() }
        else if separator(input[n-1]) {
            if n > 1 && separator(input[n-2]) { normalized_prefix(input, n-1) }
            else { normalized_prefix(input, n-1).push_back(45u8) }
        } else { normalized_prefix(input, n-1).push_back(lowercase(input[n-1])) }
    }
}

/// Every prefix of canonical bytes is already its own normalization.
#[cfg(creusot)]
#[logic]
#[requires(0 <= n && n <= input.len())]
#[requires(forall<j> 0 <= j && j < n ==> alphanumeric(input[j]) && !(65u8 <= input[j] && input[j] <= 90u8) || input[j] == 45u8)]
#[requires(forall<j> 0 < j && j < n ==> input[j] != 45u8 || input[j-1] != 45u8)]
#[ensures(normalized_prefix(input, n) == input.subsequence(0, n))]
#[variant(n)]
pub fn normalized_prefix_fixed_point(input: Seq<u8>, n: Int) {
    pearlite! {
        if n > 0 {
            normalized_prefix_fixed_point(input, n-1);
            proof_assert!(input.subsequence(0, n-1).push_back(input[n-1]).ext_eq(input.subsequence(0, n)));
        }
    }
}

/// Canonical output is a fixed point of the production normalization semantics.
#[cfg(creusot)]
#[logic]
#[requires(forall<j> 0 <= j && j < input.len() ==> alphanumeric(input[j]) && !(65u8 <= input[j] && input[j] <= 90u8) || input[j] == 45u8)]
#[requires(forall<j> 0 < j && j < input.len() ==> input[j] != 45u8 || input[j-1] != 45u8)]
#[ensures(normalized_prefix(input, input.len()) == input)]
pub fn normalization_fixed_point(input: Seq<u8>) {
    pearlite! { normalized_prefix_fixed_point(input, input.len()); }
}

#[cfg_attr(creusot, ensures(result == alphanumeric(b)))]
#[expect(
    clippy::manual_range_contains,
    reason = "direct comparisons expose ASCII bounds to the verifier"
)]
fn is_alphanumeric(b: u8) -> bool {
    (b'0' <= b && b <= b'9') || (b'A' <= b && b <= b'Z') || (b'a' <= b && b <= b'z')
}

#[cfg_attr(creusot, ensures(result == separator(b)))]
fn is_separator(b: u8) -> bool {
    b == b'-' || b == b'_' || b == b'.'
}

/// Reject invalid names and return lowercase ASCII with separator runs collapsed.
/// The returned byte vector can become a String without copying its allocation.
#[must_use]
#[cfg_attr(creusot, ensures((match result { Some(_) => true, None => false }) == valid(input@)))]
#[cfg_attr(creusot, ensures(match result {
    Some(out) => out@.len() > 0 && out@.len() <= input@.len()
        && alphanumeric(out[0]) && alphanumeric(out[out@.len()-1])
        && (forall<j> 0 <= j && j < out@.len() ==> alphanumeric(out[j]) && !(65u8 <= out[j] && out[j] <= 90u8) || out[j] == 45u8)
        && (forall<j> 0 < j && j < out@.len() ==> out[j] != 45u8 || out[j-1] != 45u8)
        && out@ == normalized_prefix(input@, input@.len())
        && normalized_prefix(out@, out@.len()) == out@,
    None => true,
}))]
#[expect(
    clippy::manual_range_contains,
    reason = "direct comparisons expose ASCII bounds to the verifier"
)]
pub fn normalize(input: &[u8]) -> Option<Vec<u8>> {
    if input.is_empty() || !is_alphanumeric(input[0]) || !is_alphanumeric(input[input.len() - 1]) {
        return None;
    }
    let mut i = 0usize;
    #[cfg_attr(creusot, invariant(i@ <= input@.len()))]
    #[cfg_attr(creusot, invariant(forall<j> 0 <= j && j < i@ ==> alphanumeric(input[j]) || separator(input[j])))]
    #[cfg_attr(creusot, variant(input@.len() - i@))]
    while i < input.len() {
        if !is_alphanumeric(input[i]) && !is_separator(input[i]) {
            return None;
        }
        i += 1;
    }
    let mut output = Vec::new();
    let mut i = 0usize;
    let mut previous_separator = false;
    #[cfg_attr(creusot, invariant(i@ <= input@.len()))]
    #[cfg_attr(creusot, invariant(output@.len() <= i@))]
    #[cfg_attr(creusot, invariant(output@ == normalized_prefix(input@, i@)))]
    #[cfg_attr(creusot, invariant(previous_separator == (i@ > 0 && separator(input[i@-1]))))]
    #[cfg_attr(creusot, invariant((i@ == 0) == (output@.len() == 0)))]
    #[cfg_attr(creusot, invariant(i@ > 0 ==> alphanumeric(output[0])))]
    #[cfg_attr(creusot, invariant(i@ > 0 ==> (output[output@.len()-1] == 45u8) == previous_separator))]
    #[cfg_attr(creusot, invariant(forall<j> 0 <= j && j < output@.len() ==> alphanumeric(output[j]) && !(65u8 <= output[j] && output[j] <= 90u8) || output[j] == 45u8))]
    #[cfg_attr(creusot, invariant(forall<j> 0 < j && j < output@.len() ==> output[j] != 45u8 || output[j-1] != 45u8))]
    #[cfg_attr(creusot, variant(input@.len() - i@))]
    while i < input.len() {
        let b = input[i];
        if is_separator(b) {
            if !previous_separator {
                output.push(b'-');
            }
            previous_separator = true;
        } else {
            output.push(if b'A' <= b && b <= b'Z' { b + 32 } else { b });
            previous_separator = false;
        }
        i += 1;
    }
    #[cfg(creusot)]
    proof_assert! { normalization_fixed_point(output@); true }
    Some(output)
}
