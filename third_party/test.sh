test_cases=`ls`

for i in ${test_cases[@]}
do
	if [ "${i}" != 'serde-rs' ]; then
		cd ${i} && xargo build --target x86_64-unknown-linux-sgx --release && git clean -fxd && cd ..
	fi
done

echo "Done!"
