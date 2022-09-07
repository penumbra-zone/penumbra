let liveViewSettings = {
    animationDuration: 250,
    animationEasing: d3.easeExpInOut,
    renderInterval: 500,
    pollRetry: 1000,
    pollDelay: 0,
    initialPrecision: 1,
    precisionDecreaseThreshold: 100,
    precisionDecreaseFactor: 1.5,
};

function transition() {
    return d3.transition("main")
        .ease(liveViewSettings.animationEasing)
        .duration(liveViewSettings.animationDuration);
}

let graphviz = d3.select("#graph").graphviz()
    .transition(transition)
    .growEnteringEdges(false) // d3-graphviz bug: if enabled, this causes an error
    .tweenShapes(false) // Increases performance
    .logEvents(false) // Disabling logging increases performance
    .on("initEnd", run);

function run() {
    // Initial state
    let changed = false;
    let renderedRecently = false;
    let precision = liveViewSettings.initialPrecision;
    let latestDot = 'digraph {}';
    let next = false;
    let forgotten = 0;
    let position = {
        epoch: 0,
        block: 0,
        commitment: 0,
    };

    // Long-poll loop to get the latest dot render of the tree
    function poll() {
        let query_string = "?epoch=" + position.epoch + "&block=" + position.block + "&commitment=" + position.commitment + "&forgotten=" + forgotten + "&next=" + next;
        let url = window.location.href + "/dot" + query_string;

        d3.json(url).then(response => {
            latestDot = response.graph;
            forgotten = response.forgotten;
            position = response.position;
            next = true;
            changed = true;
            // Start a new render task, if one isn't already in progress
            setTimeout(render, 0);
            // Schedule the polling to recur
            setTimeout(poll, liveViewSettings.pollDelay);
        }).catch(error => {
            console.log(error);
            setTimeout(poll, liveViewSettings.pollRetry);
        });
    }

    // Render the current dot, if it has changed, and continue rendering until it hasn't changed
    // from underneath us while rendering
    function render() {
        if (changed && !renderedRecently) {
            // Mark the render as having started, so other calls to render will stop if there
            // haven't been other updates
            changed = false;
            // Mark the render as being recent, and schedule the recency to expire after the
            // recency duration
            renderedRecently = true;
            setTimeout(() => {
                // After the recency duration, mark the render as no longer being recent, and
                // re-render if anything has changed
                renderedRecently = false;
                render();
            }, liveViewSettings.renderInterval);
            // Render the graph
            let start = performance.now();
            graphviz.tweenPrecision(precision + "%").dot(latestDot, () => {
                // If the pre-calculation took too long, decrease the tweening precision
                let end = performance.now();
                let elapsed = end - start;
                if (elapsed > liveViewSettings.precisionDecreaseThreshold) {
                    precision = precision * liveViewSettings.precisionDecreaseFactor;
                } else if (elapsed < liveViewSettings.precisionDecreaseThreshold * liveViewSettings.precisionDecreaseFactor) {
                    precision = precision / liveViewSettings.precisionDecreaseFactor;
                }
            }).render(render);
        }
    }

    poll();
}