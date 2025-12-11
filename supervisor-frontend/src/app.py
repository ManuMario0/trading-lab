import streamlit as st
import zmq

st.set_page_config(page_title="Trading Bot Supervisor", layout="wide")

st.title("Trading Bot Supervisor")

st.sidebar.header("Controls")
if st.sidebar.button("Kill Engine"):
    st.error("Kill signal sent (Not implemented)")

st.header("Portfolio Status")
st.write("Waiting for data...")

# TODO: Connect ZMQ SUB socket to listen to Multiplexer/Engine
