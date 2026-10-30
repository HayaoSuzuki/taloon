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

end TaloonOracle
