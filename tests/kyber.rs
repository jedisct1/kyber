use kyber;
use rand;

use kyber::params::{CIPHERTEXTBYTES, PUBLICKEYBYTES, SECRETKEYBYTES, SYMBYTES};

#[test]
fn test_kyber() {
    let mut key_a = [0; SYMBYTES];
    let mut key_b = [0; SYMBYTES];
    let mut pk = [0; PUBLICKEYBYTES];
    let mut sendb = [0; CIPHERTEXTBYTES];
    let mut sk_a = [0; SECRETKEYBYTES];
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        kyber::kem::keypair(&mut rng, &mut pk, &mut sk_a);
        kyber::kem::enc(&mut rng, &mut sendb, &mut key_b, &pk);
        kyber::kem::dec(&mut key_a, &sendb, &sk_a);

        assert!(key_a.iter().any(|&n| n != 0));
        assert_eq!(key_a, key_b);
    }
}
