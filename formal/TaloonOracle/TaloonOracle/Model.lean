import Std

namespace TaloonOracle

inductive Expression where
  | atom (unit : String)
  | both (left right : Expression)
  | either (left right : Expression)
  deriving Repr, DecidableEq

def evaluate : Expression → (String → Bool) → Bool
  | .atom unit, allowed => allowed unit
  | .both left right, allowed => evaluate left allowed && evaluate right allowed
  | .either left right, allowed => evaluate left allowed || evaluate right allowed

def validThrough (today expiry : Nat) : Bool := decide (today ≤ expiry)

def sufficient (archive required : Bool) : Bool := archive || !required

structure Observation where
  request : Nat
  archive : Bool
  permitted : Bool
  deriving Repr, DecidableEq

def complete (expected : List Nat) (observations : List Observation) (required : Bool) : Bool :=
  expected.all (fun id => (observations.filter (fun o => o.request == id)).length == 1 &&
    observations.any (fun o => o.request == id && sufficient o.archive required)) &&
  observations.all (fun o => expected.contains o.request)

def exitCode (isComplete violation warnings denyWarnings : Bool) : Nat :=
  if !isComplete then 2 else if violation || (warnings && denyWarnings) then 1 else 0

end TaloonOracle
