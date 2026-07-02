# Cap'n Proto schema for the Clade event bus — the M1 wire format.
#
# M0 ships newline-delimited JSON (see src/lib.rs) so the scaffold builds
# without the capnp toolchain. This schema is the committed shape of the
# swap; codegen lands with M1 and replaces the JSON framing inside
# clade-proto without touching any service.

@0xd4c53a9fe2c1ade0;

struct Event {
  union {
    serviceUp   :group { service @0 :Text; pid @1 :UInt32; }
    heartbeat   :group { service @2 :Text; uptimeSecs @3 :UInt64; }
    weaveReady  :group { unused @4 :Void; }
    serviceDown :group { service @5 :Text; code @6 :Int32; hasCode @7 :Bool; }
  }
}
