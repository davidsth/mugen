
require "./ui"
include UI

ui = Container.new("root") do |c|
  c << Label.new("Hello World", "greeting")
  c << Button.new("Click Me", "click_me")
end

File.write("layout.json", JSON.build do |json|
  ui.to_json(json)
end)
