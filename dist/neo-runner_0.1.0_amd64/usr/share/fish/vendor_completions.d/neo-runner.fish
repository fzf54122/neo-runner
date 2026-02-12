# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_neo_runner_global_optspecs
	string join \n version f/file= output= h/help
end

function __fish_neo_runner_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_neo_runner_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_neo_runner_using_subcommand
	set -l cmd (__fish_neo_runner_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c neo-runner -n "__fish_neo_runner_needs_command" -s f -l file -r
complete -c neo-runner -n "__fish_neo_runner_needs_command" -l output -r -f -a "text\t''
json\t''"
complete -c neo-runner -n "__fish_neo_runner_needs_command" -l version
complete -c neo-runner -n "__fish_neo_runner_needs_command" -s h -l help -d 'Print help'
complete -c neo-runner -n "__fish_neo_runner_needs_command" -f -a "run"
complete -c neo-runner -n "__fish_neo_runner_needs_command" -f -a "plan"
complete -c neo-runner -n "__fish_neo_runner_needs_command" -f -a "validate"
complete -c neo-runner -n "__fish_neo_runner_needs_command" -f -a "completion"
complete -c neo-runner -n "__fish_neo_runner_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c neo-runner -n "__fish_neo_runner_using_subcommand run" -s f -l file -r
complete -c neo-runner -n "__fish_neo_runner_using_subcommand run" -l output -r -f -a "text\t''
json\t''"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand run" -s h -l help -d 'Print help'
complete -c neo-runner -n "__fish_neo_runner_using_subcommand plan" -s f -l file -r
complete -c neo-runner -n "__fish_neo_runner_using_subcommand plan" -l output -r -f -a "text\t''
json\t''"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand plan" -s h -l help -d 'Print help'
complete -c neo-runner -n "__fish_neo_runner_using_subcommand validate" -s f -l file -r
complete -c neo-runner -n "__fish_neo_runner_using_subcommand validate" -l output -r -f -a "text\t''
json\t''"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand validate" -s h -l help -d 'Print help'
complete -c neo-runner -n "__fish_neo_runner_using_subcommand completion" -s f -l file -r
complete -c neo-runner -n "__fish_neo_runner_using_subcommand completion" -l output -r -f -a "text\t''
json\t''"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand completion" -s h -l help -d 'Print help'
complete -c neo-runner -n "__fish_neo_runner_using_subcommand help; and not __fish_seen_subcommand_from run plan validate completion help" -f -a "run"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand help; and not __fish_seen_subcommand_from run plan validate completion help" -f -a "plan"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand help; and not __fish_seen_subcommand_from run plan validate completion help" -f -a "validate"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand help; and not __fish_seen_subcommand_from run plan validate completion help" -f -a "completion"
complete -c neo-runner -n "__fish_neo_runner_using_subcommand help; and not __fish_seen_subcommand_from run plan validate completion help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
