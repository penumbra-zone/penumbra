let latest_dot = 'digraph {}';
let next = false;
let forgotten = 0;
let position = {
    epoch: 0,
    block: 0,
    commitment: 0,
};
let log = true;

function transition() {
    return d3.transition("main")
        .ease(d3.easeExpInOut)
        .duration(250);
}

let graphviz = d3.select("#graph").graphviz()
    .transition(transition)
    .growEnteringEdges(false) // d3-graphviz bug: if enabled, this causes an error
    .tweenShapes(false) // Increases performance
    .tweenPrecision("2%") // Increases performance over default of "1pt"
    .logEvents(true)
    .on("initEnd", poll_loop);

function poll_loop() {
    let query_string = "?epoch=" + position.epoch + "&block=" + position.block + "&commitment=" + position.commitment + "&forgotten=" + forgotten + "&next=" + next;
    let url = window.location.href + "/dot" + query_string;

    let xhr = new XMLHttpRequest();
    xhr.open("GET", url);
    xhr.send();
    xhr.onload = () => {
        if (xhr.status === 200) {
            let response = JSON.parse(xhr.responseText);
            latest_dot = response.graph;
            forgotten = response.forgotten;
            position = response.position;
            next = true;
            graphviz.renderDot(latest_dot).on("end", poll_loop);
        } else {
            console.log("Error: " + xhr.responseText);
        }
    };
}