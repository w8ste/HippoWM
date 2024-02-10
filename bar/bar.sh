#!/bin/bash

# Function to update the Lemonbar content
update_bar() {
    echo "%{c}$(date +"%A %B %d, %Y %I:%M %p")"  # Center-aligned date and time
}

# Start Lemonbar and continuously update its content
update_bar | lemonbar -p -g 1920x30+0+0 -B "#333333" -F "#ffffff" -f "Monospace:size=10" &

# Continuously update Lemonbar content every second
while true; do
    update_bar
    sleep 1
done


