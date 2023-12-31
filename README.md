# A pallet, that would take a series of encoded bytes and convert them to volatile side effects for further interactions

1. Build pallet -> `cargo build`
2. Run the test script -> `cargo test`
3. Run the test script with the logging enabled `RUST_LOG=info cargo t -- --nocapture`

### Extrinsics supported

```rust
/// Generate volatile side effects steps from encoded bytes provided
pub fn generate_side_effects_steps(
            origin: OriginFor<T>,
            encoded_bytes: Vec<u8>,
) -> DispatchResult 

/// Commit a number of side effects steps under given execution id
pub fn commit(
        origin: OriginFor<T>,
        actor_id: T::ActorId,
        execution_id: T::ExecutionId,
        number_of_steps: u32,
    ) -> DispatchResult 

/// Revert commited side effects under given execution id
pub fn revert(
        origin: OriginFor<T>,
        actor_id: T::ActorId,
        execution_id: T::ExecutionId,
    ) -> DispatchResult
```

### An example of input/output for `generate_side_effects_steps` extrinsic

    Input bytes --> 
        [
            2, 124, 1, 124, 7, 124, 92, 85, 23, 125, 103, 176, 100, 187, 93, 24, 154, 62, 29, 218, 217, 188, 102, 70, 224, 46, 100, 214, 227, 8, 245, 172, 187, 21, 51, 172, 67, 13, 124, 60, 85, 23, 125, 103, 176, 100, 187, 93, 24, 154, 62, 29, 218, 217, 188, 102, 70, 224, 46, 100, 214, 227, 8, 245, 172, 187, 21, 51, 172, 67, 13, 124, 0, 0, 0, 0, 0, 7, 161, 32, 124, 1, 59, 1, 124, 6, 124, 60, 85, 23, 125, 103, 176, 100, 187, 93, 24, 154, 62, 29, 218, 217, 188, 102, 70, 224, 46, 100, 214, 227, 8, 245, 172, 187, 21, 51, 172, 67, 13, 124, 156, 85, 23, 125, 103, 176, 100, 187, 93, 24, 154, 62, 29, 218, 217, 188, 102, 70, 224, 46, 100, 214, 227, 8, 245, 172, 187, 21, 51, 172, 67, 13, 124, 0, 0, 0, 0, 0, 7, 161, 32
        ]

    Side effect parameters bytes --> [
        [
            2,
        ],
        [
            1,
        ],
        [
            7,
        ],
        [
            92,
            85,
            23,
            125,
            103,
            176,
            100,
            187,
            93,
            24,
            154,
            62,
            29,
            218,
            217,
            188,
            102,
            70,
            224,
            46,
            100,
            214,
            227,
            8,
            245,
            172,
            187,
            21,
            51,
            172,
            67,
            13,
        ],
        [
            60,
            85,
            23,
            125,
            103,
            176,
            100,
            187,
            93,
            24,
            154,
            62,
            29,
            218,
            217,
            188,
            102,
            70,
            224,
            46,
            100,
            214,
            227,
            8,
            245,
            172,
            187,
            21,
            51,
            172,
            67,
            13,
        ],
        [
            0,
            0,
            0,
            0,
            0,
            7,
            161,
            32,
        ],
        [
            1,
        ],
    ]

    Side effect --> MultiChain {
        chain_from: Kusama,
        chain_to: Polkadot,
        side_effect: SideEffect {
            execution_type: MultiTran {
                caller: 5c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d (5E9mYH6j...),
                to: 3c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d (5DRp1QQY...),
                amount: 500000,
                asset: Asset(
                    Polkadot,
                ),
            },
            state: Submitted,
        },
    }

    Side effect parameters bytes --> [
        [
            1,
        ],
        [
            6,
        ],
        [
            60,
            85,
            23,
            125,
            103,
            176,
            100,
            187,
            93,
            24,
            154,
            62,
            29,
            218,
            217,
            188,
            102,
            70,
            224,
            46,
            100,
            214,
            227,
            8,
            245,
            172,
            187,
            21,
            51,
            172,
            67,
            13,
        ],
        [
            156,
            85,
            23,
            125,
            103,
            176,
            100,
            187,
            93,
            24,
            154,
            62,
            29,
            218,
            217,
            188,
            102,
            70,
            224,
            46,
            100,
            214,
            227,
            8,
            245,
            172,
            187,
            21,
            51,
            172,
            67,
            13,
        ],
        [
            0,
            0,
            0,
            0,
            0,
            7,
            161,
            32,
        ],
    ]

    Side effect --> SingleChain {
        chain: Polkadot,
        side_effect: SideEffect {
            execution_type: Tran {
                caller: 3c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d (5DRp1QQY...),
                to: 9c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d (5Fbgc2V5...),
                amount: 500000,
            },
            state: Submitted,
        },
    }
