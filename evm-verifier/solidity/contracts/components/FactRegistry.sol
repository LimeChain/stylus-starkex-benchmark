// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.6.12;


/*
❯ forge inspect evm-verifier/solidity/contracts/components/FactRegistry.sol:FactRegistry abi
╭----------+-----------------------------------------+------------╮
| Type     | Signature                               | Selector   |
+=================================================================+
| function | hasRegisteredFact() view returns (bool) | 0xd6354e15 |
|----------+-----------------------------------------+------------|
| function | isValid(bytes32) view returns (bool)    | 0x6a938567 |
╰----------+-----------------------------------------+------------╯
*/
contract FactRegistry  {
    // Mapping: fact hash -> true.
    mapping(bytes32 => bool) private verifiedFact;

    // Indicates whether the Fact Registry has at least one fact registered.
    bool anyFactRegistered;

    /*
      Checks if a fact has been verified.
    */
    function isValid(bytes32 fact) external view  returns (bool) {
        return _factCheck(fact);
    }

    /*
      This is an internal method to check if the fact is already registered.
      In current implementation of FactRegistry it's identical to isValid().
      But the check is against the local fact registry,
      So for a derived referral fact registry, it's not the same.
    */
    function _factCheck(bytes32 fact) internal view returns (bool) {
        return verifiedFact[fact];
    }

    function registerFact(bytes32 factHash) internal {
        // This function stores the fact hash in the mapping.
        verifiedFact[factHash] = true;

        // Mark first time off.
        if (!anyFactRegistered) {
            anyFactRegistered = true;
        }
    }

    /*
      Indicates whether at least one fact was registered.
    */
    function hasRegisteredFact() external view  returns (bool) {
        return anyFactRegistered;
    }
}
