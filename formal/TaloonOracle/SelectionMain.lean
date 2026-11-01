import TaloonOracle.Selection
import Lean.Data.Json
open TaloonOracle.Selection

def graphs : List (List Edge) := [
  [], [⟨0,1,.normal⟩,⟨1,2,.normal⟩],
  [⟨0,1,.normal⟩,⟨1,0,.normal⟩], [⟨0,1,.extra⟩,⟨1,2,.extra⟩],
  [⟨0,1,.development⟩,⟨1,2,.normal⟩],
  [⟨0,3,.normal⟩,⟨3,2,.extra⟩],
  [⟨0,1,.normal⟩,⟨0,2,.extra⟩,⟨1,2,.normal⟩],
  [⟨1,2,.normal⟩,⟨2,1,.extra⟩],
  [⟨0,0,.extra⟩,⟨0,1,.normal⟩,⟨0,1,.extra⟩],
  [⟨3,1,.development⟩] ]
def kindString : Kind → String
  | .normal => "normal" | .extra => "extra" | .development => "development"
def edgeJson (e : Edge) : Lean.Json := Lean.Json.mkObj [
  ("source",Lean.toJson e.source),("target",Lean.toJson e.target),("kind",.str (kindString e.kind))]
def sensitive : Bool :=
  select [0,1,2,3] [3] [0] [⟨0,1,.extra⟩] == [0,1] &&
  select [0,1,2,3] [3] [0] [⟨0,1,.development⟩] == [0] &&
  select [0,1,2,3] [3] [3] [] == [0,1,2] &&
  select [0,1,2,3] [3] [0,1] [] == [0,1]
example : sensitive = true := by decide

def corpus : String := Id.run do
  let mut rows : List String := []
  let mut number := 0
  for edges in graphs do
    for roots in [[0],[0,1],[3],[0,3],[]] do
      rows := rows ++ [(Lean.Json.mkObj [
        ("schema",Lean.toJson (1:Nat)),("id",.str s!"selection-{number}"),
        ("edges",Lean.toJson (edges.map edgeJson)),("roots",Lean.toJson roots),
        ("expected",Lean.toJson (select [0,1,2,3] [3] roots edges))]).compress]
      number := number + 1
  return String.intercalate "\n" rows ++ "\n"

def main (args : List String) : IO UInt32 := do
  unless sensitive do
    IO.eprintln "selection sensitivity failed"; return 2
  match args with
  | ["--output",path] => IO.FS.writeFile path corpus; return 0
  | ["--check",path] =>
      if (← IO.FS.readFile path) == corpus then return 0
      IO.eprintln "stale selection corpus"; return 1
  | [] => IO.print corpus; return 0
  | _ => IO.eprintln "usage: generate_selection [--output PATH | --check PATH]"; return 2
