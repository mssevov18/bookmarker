function b
    if test (count $argv) -eq 0
        bookmarker all-top
        echo -n "Choose key: "
        read key

        set path (bookmarker quick $key)
        set code $status

        if test $code -eq 0
            set -gx PRE_BOOKJUMP_DIR (pwd)
            cd "$path"; or echo "❌ Failed to cd"
        else
            return $code
        end
        return
    end

    switch $argv[1]
        case add remove purge clean all all-top all-long quick help --help -h --version
            bookmarker $argv
            return
        case '*'
            set path (bookmarker quick $argv[1])
            set code $status

            if test $code -eq 0
                set -gx PRE_BOOKJUMP_DIR (pwd)
                cd "$path"; or begin
                    echo "❌ Failed to cd into '$path'" >&2
                    return 1
                end
            else
                return $code
            end
    end
end

function bb
    if set -q PRE_BOOKJUMP_DIR
	set temp_bookjump_dir (pwd)
        cd "$PRE_BOOKJUMP_DIR"; or cd "$HOME"
	set -gx PRE_BOOKJUMP_DIR "$temp_bookjump_dir"
    else
        cd "$HOME"
    end
end
