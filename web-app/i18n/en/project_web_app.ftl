hello-world = Hello World!

time_unit_count =
    {$count ->
        [one] 1 {$unit}
       *[other] {$count} {$unit}s
    }