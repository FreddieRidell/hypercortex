import * as R from "ramda";
import os from "os";

import getCortexDb from "@hypercortex/hypercortex-cli-client";
import createTask from "@hypercortex/object-type-task";
import createTelemetry from "@hypercortex/object-type-telemetry";

import partitionCommandsAndArgs from "./parseArgs";

import add from "./commands/add";
import done from "./commands/done";
import snooze from "./commands/snooze";
import basicDisplay from "./commands/basicDisplay";

const commandToFunction = { add, done, snooze };

const main = async () => {
	const db = await getCortexDb();

	const { task, taskAll } = createTask(db);
	const { telemetry } = createTelemetry(db);

	telemetry(db.local.key.toString("hex")).nameSet(os.hostname());

	const { filter, command, modifications } = partitionCommandsAndArgs(
		commandToFunction,
	)(process.argv);

	await (commandToFunction[command] || basicDisplay)({
		filter,
		modifications,
		taskAll,
		task,
	});
};

try {
	main();
} catch (e) {
	console.error(e);
}
