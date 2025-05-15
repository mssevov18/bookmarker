b() {
	if [ $# -eq 0 ]; then
		bookmarker all-top
		echo -n "Choose key: "
		read -r key

		local path
		path=$(bookmarker quick "$key")
		local code=$?

		if [ $code -eq 0 ]; then
			export PRE_BOOKJUMP_DIR="$PWD"
			cd "$path" || echo "❌ Failed to cd"
		else
			return $code
		fi
		return
	fi

	case "$1" in
	add | remove | purge | clean | all | all-top | all-long | quick | help | --help | -h | --version)
		bookmarker "$@"
		return
		;;
	*)
		local path
		path=$(bookmarker quick "$1")
		local code=$?

		if [ $code -eq 0 ]; then
			export PRE_BOOKJUMP_DIR="$PWD"
			cd "$path" || {
				echo "❌ Failed to cd into '$path'" >&2
				return 1
			}
		else
			return $code
		fi
		;;
	esac
}

bb() {
	if [ -n "$PRE_BOOKJUMP_DIR" ]; then
		local temp_bookjump_dir="$PWD"
		cd "$PRE_BOOKJUMP_DIR" || cd "$HOME"
		export PRE_BOOKJUMP_DIR="$temp_bookjump_dir"
	else
		cd "$HOME"
	fi
}
