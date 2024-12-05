# An e-ink RP 2040 desk display 

 This is a RP 2040 e-ink desk display with a firmware written in rust using embassy. The goal is to make this my most "professional" product.

 Goals:
 - A well organized firmware taking advantage of embassy's tasks and events as shown [here](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/orchestrate_tasks.rs). 
 - A custom designed PCB for the product
 - 100% open source. 

This is not a product I am planning on selling, but more so for personal use. Although the project is setup so anyone can use it and customize to an extent. I just want to take a project all the way through and have a nice finished product. 

Features:
- Time and date display
- 5 day forecast for your location
- Get the current weather for your location 
- Read Co2, Temperature and Humidity from the SCD-40 sensor
- At work we use a clock in/out software and I always forget to clock back in after lunch so will show that status. 
- possibly battery powered 
- If battery powered "advanced" power savings by an external RTC



# Special Thanks
- [Weather Icons](https://github.com/manifestinteractive/weather-underground-icons)
