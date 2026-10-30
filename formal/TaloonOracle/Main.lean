import TaloonOracle.Proofs
import Lean.Data.Json

open TaloonOracle

def renderExpression : Expression → String
  | .atom unit => unit
  | .both left right => "(" ++ renderExpression left ++ " AND " ++ renderExpression right ++ ")"
  | .either left right => "(" ++ renderExpression left ++ " OR " ++ renderExpression right ++ ")"

def expressions : List Expression := [
  .atom "MIT", .both (.atom "MIT") (.atom "Apache-2.0"),
  .either (.atom "MIT") (.atom "Apache-2.0"),
  .atom "Apache-2.0 WITH LLVM-exception", .atom "GPL-2.0-only",
  .atom "GPL-2.0-or-later",
  .both (.atom "MIT") (.either (.atom "Apache-2.0") (.atom "BSD-3-Clause")) ]

def allowedSets : List (List String) := [[], ["MIT"], ["Apache-2.0"],
  ["MIT", "Apache-2.0"], ["Apache-2.0 WITH LLVM-exception"], ["GPL-2.0-only"]]

-- Deliberately broken variants stay outside imported semantic modules.
def brokenOr (left right : Bool) := left && right
def brokenAnd (left right : Bool) := left || right
def brokenExpiry (today expiry : Nat) : Bool := decide (today < expiry)
def sensitivity : Bool :=
  brokenOr true false != (true || false) && brokenAnd true false != (true && false) &&
    brokenExpiry 30 30 != validThrough 30 30

example : sensitivity = true := by decide

def corpus : String := Id.run do
  let mut rows : List String := []
  let mut n := 0
  for expression in expressions do
    for allowed in allowedSets do
      rows := rows ++ [(Lean.Json.mkObj [
        ("schema", Lean.toJson (1 : Nat)), ("id", .str s!"permission-{n}"),
        ("mode", .str "strict"), ("kind", .str "permission"),
        ("expression", .str (renderExpression expression)),
        ("allow", Lean.toJson allowed),
        ("expected", Lean.toJson (evaluate expression (fun unit => allowed.contains unit))) ]).compress]
      n := n + 1
  for expiry in [29, 30, 31] do
    rows := rows ++ [(Lean.Json.mkObj [
      ("schema", Lean.toJson (1 : Nat)), ("id", .str s!"expiry-{expiry}"),
      ("mode", .str "strict"), ("kind", .str "expiry"),
      ("date", .str "2026-10-30"), ("expires", .str s!"2026-10-{expiry}"),
      ("expected", Lean.toJson (validThrough 30 expiry)) ]).compress]
  return String.intercalate "\n" rows ++ "\n"

def main (args : List String) : IO UInt32 := do
  unless sensitivity do
    IO.eprintln "broken-variant sensitivity failed"
    return 2
  match args with
  | ["--output", path] => IO.FS.writeFile path corpus; return 0
  | ["--check", path] =>
      let actual ← IO.FS.readFile path
      if actual == corpus then return 0
      IO.eprintln "stale Lean corpus"
      return 1
  | ["--stats"] =>
      IO.println s!"permission_cases={expressions.length * allowedSets.length} expiry_cases=3 sensitivity={sensitivity}"
      return 0
  | [] => IO.print corpus; return 0
  | _ => IO.eprintln "usage: generate_corpus [--output PATH | --check PATH | --stats]"; return 2
