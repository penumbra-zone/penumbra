function keyControl() {
    let actions = [];
    let pendingCount = null;

    const queries = {
        'c': 'insert?witness=forget',
        'C': 'insert?witness=keep',
        'b': 'end-block',
        'B': 'insert-block-root',
        'e': 'end-epoch',
        'E': 'insert-epoch-root',
        'f': 'forget',
        'n': 'new',
    };

    const digits = new Set(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    const stops = new Set(['Escape', 'Backspace', 'Enter', 'q', 'Q']);

    window.addEventListener('keydown', event => {
        let key = event.key;

        if (stops.has(key) || (key === 'c' && event.ctrlKey)) {
            event.preventDefault();
            actions = [];
            pendingCount = null;
        }  else if (key in queries) {
            event.preventDefault();

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
        }
    });

    function doAction() {
        if (actions.length === 0) {
            return;
        }

        let action = actions[actions.length - 1];
        let key = action.key;
        if (action.count === 0) {
            // This action is done
            actions.pop();
            return;
        } else {
            // Decrement the count
            action.count -= 1;
        }

        let url = window.location.origin + '/' + queries[key];

        d3.text(url, {method: "post"}).then(() => {
            // Continue doing actions until none are left to do
            doAction();
        }).catch(error => {
            // If there was an error, stop the loop
            console.log(error);
        });
    }
}

keyControl();