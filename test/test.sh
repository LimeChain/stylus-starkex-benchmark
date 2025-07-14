pk=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
rpc_url=http://localhost:8547

PROOF_PARAMS=$(tr '\n' ' ' < ../inputs/proof_params.txt)
PROOF=$(tr '\n' ' ' < ../inputs/proof.txt)
PUBLIC_INPUT=$(tr '\n' ' ' < ../inputs/public_input.txt)

cpu_contract=0xf1fede8133b032a1ebd78e107d510faec3e51365
actual_output=$(cast call $cpu_contract "verifyProofExternal(uint256[],uint256[],uint256[])" \
	$PROOF_PARAMS \
	$PROOF \
	$PUBLIC_INPUT \
	--rpc-url $rpc_url --private-key $pk -vvv)

expected_output=$(tr '\n' ' ' < ../inputs/expected_out.txt)

if [[ "$actual_output" == "$expected_output" ]]; then
  echo ""
  echo "----------CORRECT----------"
else
  echo ""
  echo "----------NEGATIVE!----------"
  exit 1 # Exit with a non-zero status to indicate failure
fi