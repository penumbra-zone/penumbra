const liveViewSettings = {
    animationDuration: 200,
    animationEasing: d3.easeExpInOut,
    renderInterval: 450,
    pollRetry: 1000,
    longPollDelay: 0,
    initialPrecision: 1,
    precisionDecreaseThreshold: 200,
    precisionDecreaseFactor: 1.5,
};

function transition() {
    return d3.transition("main")
        .ease(liveViewSettings.animationEasing)
        .duration(liveViewSettings.animationDuration);
}

const graphviz = d3.select("#graph").graphviz()
    .transition(transition)
    .growEnteringEdges(false) // d3-graphviz bug: if enabled, this causes an error
    .tweenShapes(false) // Increases performance
    .logEvents(false) // Disabling logging increases performance
    // Set the SVG to fill the window
    .width(window.innerWidth)
    .height(window.innerHeight)
     // Start the event loop once the graphviz stuff is loaded
    .on("initEnd", run);

function run() {
    // When the window is resized, resize the graphviz SVG also
    window.addEventListener("resize", () => {
        // Immediately resize it
        d3.select("#graph").select("svg")
            .width(window.innerWidth)
            .height(window.innerHeight);
        // Resize it in all future renders
        graphviz
            .width(window.innerWidth)
            .height(window.innerHeight);
    });

    // Initial state
    let changed = false;
    let renderedRecently = false;
    let precision = liveViewSettings.initialPrecision;
    let latestDot = 'digraph {}';
    let forgotten = 0;
    let position = {
        epoch: 0,
        block: 0,
        commitment: 0,
    };

    // Long-poll loop to get the latest dot render of the tree
    function poll(long) {
        let query_string = "";
        if (long) {
            query_string = "?epoch=" + position.epoch + "&block=" + position.block + "&commitment=" + position.commitment + "&forgotten=" + forgotten + "&next=" + long;
        }
        let url = window.location.href + "/dot" + query_string;

        d3.json(url).then(response => {
            latestDot = response.graph;
            forgotten = response.forgotten;
            position = response.position;
            changed = true;
            // Start a new render task, if one isn't already in progress
            setTimeout(render, 0);
            // Schedule the polling to recur
            if (long) {
                setTimeout(() => poll(long), liveViewSettings.longPollDelay);
            }
        }).catch(error => {
            console.log(error);
            setTimeout(() => poll(long), liveViewSettings.pollRetry);
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

    // Do one initial short-poll to get the current state of the graph
    poll(true);

    // Start the long-poll loop over non-interior changes
    poll(false);

    // Interior mutation caused by evaluating the lazy frontier hashes won't cause the position or
    // forgotten index to advance, so it won't be caught by the long-poll loop: we use the SSE
    // endpoint to monitor for these changes, and trigger an immediate short poll when they occur
    const changes = new EventSource(window.location.href + "/changes");
    changes.addEventListener("changed", (event) => {
        // When a change occurs, check to see if *nothing has changed* about the position and
        // forgotten count: only then, do a short poll to get the latest dot.
        let response = JSON.parse(event.data);
        // Figure out whether the event was an interior mutation, and only do a short-poll if it was
        // an interior mutation (otherwise we'd be wasting our time because the long poll will get
        // to that change)
        if (response.position === null && position != null) {
            return;
        }
        let interior =
            response.forgotten === forgotten
            && ((response.position === null && position === null)
                || (response.position.epoch === position.epoch
                    && response.position.block === position.block
                    && response.position.commitment === position.commitment))
        if (interior) {
            poll(false);
        }
    });
}