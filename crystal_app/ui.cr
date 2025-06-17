
require "json"

module UI
  abstract class Node
    def initialize(@type : String, @id : String?, @props : Hash(String, JSON::Any), @layout : Hash(String, JSON::Any), @children : Array(Node))
    end

    def to_json(builder : JSON::Builder)
      builder.object do
        builder.field "type", @type
        builder.field "id", @id if @id
        builder.field "props", @props
        builder.field "layout", @layout
        builder.field "children" do
          builder.array do
            @children.each(&.to_json(builder))
          end
        end
      end
    end
  end

  class Container < Node
    def initialize(id = nil, layout = default_layout, &block)
      children = [] of Node
      yield children
      super("container", id, {} of String => JSON::Any, layout, children)
    end

    def default_layout
      {
        "direction" => JSON::Any.new("column"),
        "align_items" => JSON::Any.new("center"),
        "justify_content" => JSON::Any.new("start")
      }
    end
  end

  class Label < Node
    def initialize(text : String, id = nil, layout = UI.size_layout(200, 30))
      props = { "text" => JSON::Any.new(text) }
      super("label", id, props, layout, [] of Node)
    end
  end

  class Button < Node
    def initialize(text : String, id = nil, layout = UI.size_layout(120, 40))
      props = { "text" => JSON::Any.new(text) }
      super("button", id, props, layout, [] of Node)
    end
  end

  def self.size_layout(w : Int32, h : Int32)
    {
      "width" => JSON::Any.new(w),
      "height" => JSON::Any.new(h)
    }
  end
end
