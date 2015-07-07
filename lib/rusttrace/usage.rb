module Rusttrace
  class Usage
    class CallCount < FFI::Struct
      layout :count, :uint,
             :method_length, :uint,
             :method_name, :string

      def inspect
        "#<CallCount method_name=#{method_name} count=#{count}>"
      end

      def method_name
        self[:method_name]
      end

      def count
        self[:count]
      end
    end

    class Report < FFI::Struct
      include Enumerable

      layout :length, :uint,
             :call_counts, :pointer

      def each
        self[:length].times do |i|
          yield CallCount.new(self[:call_counts] + (i * CallCount.size))
        end
      end
    end

    module Rust
      extend FFI::Library

      if /darwin/ =~ RUBY_PLATFORM
        ffi_lib 'target/release/librusttrace.dylib'
      else
        ffi_lib 'target/release/librusttrace.so'
      end

      attach_function :new_usage, [], :pointer
      attach_function :record, [:pointer, :string, :string, :int, :string, :string], :void
      attach_function :report, [:pointer], Report.by_value
    end

    def initialize
      @value = 0
      @usage = Rust.new_usage
    end

    def attach
      set_trace_func proc { |event, file, line, id, binding, classname|
        Rust.record(@usage, event, file, line, id.to_s, classname.to_s)
      }
    end

    def report
      Rust.report(@usage)
    end

    private

    attr_reader :value
  end
end
