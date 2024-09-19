use demonstrate::demonstrate;
fn is_4() -> u8 {
    4
}

demonstrate! {
    describe "module" {
        use super::*;
        before {
            let four = 4;
        }

        // #[should_panic]
        // it "can fail" {
        //     assert_ne!(four, 4)
        // }

        #[tokio::test]
        async test "is returnable" -> Result<(), &'static str> {
            if is_4() == four {
                Ok(())
            } else {
                Err("It isn't 4! :o")
            }
        }
    }
}
