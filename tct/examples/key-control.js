let keyFeedback = true;

function keyControl() {
    // This can be set much higher when random concurrency is enabled (suggestion: 50000)
    const concurrencyLimit = 500;
    // Disable random concurrency for more quick but less responsive behavior
    const randomConcurrency = false;

    let actions = [];
    let pendingCount = null;
    let pendingActions = 0;

    const queries = {
        'c': ["post", 'insert?witness=forget', 'insert (keep)'],
        'C': ["post", 'insert?witness=keep', 'insert (forget)'],
        'b': ["post", 'end-block', 'end block'],
        'B': ["post", 'insert-block-root', 'insert block root'],
        'e': ["post", 'end-epoch', 'end epoch'],
        'E': ["post", 'insert-epoch-root', 'insert epoch root'],
        'f': ["post", 'forget', 'forget'],
        'n': ["post", 'new', 'new'],
        'r': ["get", 'root', 'evaluate root'],
    };

    const digits = new Set(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    const stops = new Set(['Escape', 'Backspace', 'Enter', 'q', 'Q']);

    window.addEventListener('keydown', event => {
        let key = event.key;

        if (stops.has(key)) {
            event.preventDefault();
            actions = [];
            pendingCount = null;
            display(key);
            display("");
        } else if (key === '!') {
            let currentKeyFeedback = keyFeedback;
            keyFeedback = true;
            if (currentKeyFeedback) {
                display("echo off");
            } else {
                display("echo on");
            }
            setTimeout(display(""), 1000);
            keyFeedback = !currentKeyFeedback;
        } else if (key === 'c' && event.ctrlKey) {
            event.preventDefault();
            actions = [];
            pendingCount = null;
            display('^C');
            display("");
        } else if (key in queries) {
            if (event.ctrlKey || event.altKey || event.metaKey) {
                return;
            } else {
                event.preventDefault();
            }

            // How many of this operation to do
            let count = 1;
            if (pendingCount !== null) {
                count = pendingCount;
                // We've used this pending count, so reset it
                pendingCount = null;
            }

            // Enqueue the operation
            if (actions.length === 0 || actions[0].key !== key) {
                actions.unshift({
                    key: key,
                    count: count,
                })
            } else {
                actions[0].count += count;
            }

            // Ensure the operation is done
            doAction();
        } else if (digits.has(key)) {
            event.preventDefault();

            // Add this digit to the pending count
            if (pendingCount === null) {
                pendingCount = 0;
            }
            pendingCount = pendingCount * 10 + parseInt(key);
            display(pendingCount);
        }
    });

    function doAction() {
        if (actions.length === 0) {
            if (pendingActions > 0 && message !== "...") {
                // Delay displaying the ellipsis so you can still read the last thing typed
                setTimeout(() => display("..."), 500);
            } else {
                display("");
            }
            return;
        }

        let action = actions[actions.length - 1];
        let key = action.key;

        if (action.count === 0) {
            // This action is done
            actions.pop();
            setTimeout(doAction, 0);
            return;
        } else {
            if (action.count > 1) {
                display(action.count + " " + (key.toUpperCase() === key ? '⇧' : '') + key);
            } else {
                display((key.toUpperCase() === key ? '⇧' : '') + key);
            }
            // Decrement the count
            action.count -= 1;
            pendingActions += 1;
        }

        // Determine whether we should perform the next request concurrently or wait for this one to
        // finish (this is effectively using a task pool of size `concurrencyLimit`). This is
        // randomized so that the user experience is less glitchy: as the pending actions goes up,
        // the probability of being concurrently scheduled goes down, until at 2 times the
        // concurrency limit, it's impossible to be concurrently scheduled. This leads to an
        // expected concurrency of the limit.
        let concurrently;
        if (!randomConcurrency) {
            concurrently = pendingActions < concurrencyLimit;
        } else {
            concurrently = pendingActions < Math.random() * concurrencyLimit * 2;
        }

        let url = window.location.origin + '/' + queries[key][1];

        d3.text(url, { method: queries[key][0] }).then(() => {
            // Don't repeat `doAction()` here, because then we'd wait for the request to finish;
            // instead, fire off a new request immediately, so we go as fast as possible.
            pendingActions -= 1;
            if (actions.length === 0) {
                if (pendingActions === 0) {
                    display("");
                }
            }
            // Only if we exceeded the concurrency limit should we schedule the action after this
            // one (otherwise we did it below, immediately)
            if (!concurrently) {
                doAction();
            }
        }).catch(error => {
            // If there was an error, stop the loop
            actions = [];
            pendingActions = 0;
            message.style("color", "red");
            display("");
            console.log(error);
        });

        // If we didn't exceed the concurrency limit, schedule the action immediately, without
        // waiting for another to finish
        if (concurrently) {
            doAction();
        }
    }

    // Set up the visual feedback box
    d3.select('#graph').insert("div").attr("id", "message");
    const message = d3.select('#message');
    const messageColor = "rgba(50, 50, 50, 0.5)";
    message.style("width", "100%");
    message.style("position", "absolute");
    message.style("bottom", "0");
    message.style("text-align", "center");
    message.style("font-size", "200pt");
    message.style("font-family", "Courier New");
    message.style("margin-bottom", "50pt");
    message.style("color", messageColor);

    function display(string) {
        if (keyFeedback) {
            if (string.length === 0) {
                message.transition()
                    .duration(500)
                    .delay(100)
                    .style("color", "rgba(100, 100, 100, 0.0)")
                    .end()
                    .then(() => message.text(string));
            } else {
                message.text(string);
                message.style("color", messageColor);
            }
        }
    }
}

keyControl();