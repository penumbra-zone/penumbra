function transition() {
    return d3.transition("main")
        .ease(d3.easeExpInOut)
        .duration(250);
}

let graphviz = d3.select("#graph").graphviz()
    .transition(transition)
    .growEnteringEdges(false) // d3-graphviz bug: if enabled, this causes an error
    .tweenShapes(false) // Increases performance
    .tweenPrecision("1%") // Increases performance over default of "1pt"
    .logEvents(false) // Disabling logging increases performance
    .on("initEnd", run);

function run() {
    let latest_dot = 'digraph {}';
    let next = false;
    let forgotten = 0;
    let position = {
        epoch: 0,
        block: 0,
        commitment: 0,
    };

    function poll_loop() {
        let query_string = "?epoch=" + position.epoch + "&block=" + position.block + "&commitment=" + position.commitment + "&forgotten=" + forgotten + "&next=" + next;
        let url = window.location.href + "/dot" + query_string;

        d3.json(url).then(response => {
            latest_dot = response.graph;
            forgotten = response.forgotten;
            position = response.position;
            next = true;
            graphviz.renderDot(latest_dot).on("end", poll_loop);
        }).catch(alert);
    }

    poll_loop();
}