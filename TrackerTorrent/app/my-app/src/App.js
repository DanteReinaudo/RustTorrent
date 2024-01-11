import * as React from 'react';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import axios from "axios";
import BarGraph from './BarGraph';
import { MOCK_RESPONSE, MOCK_RESPONSE as MOCK_TORRENT } from './mock';

interface TabPanelProps {
    children?: React.ReactNode;
    index: number;
    value: number;
}

function TabPanel(props: TabPanelProps) {
    const { children, value, index, ...other } = props;

    return (
        <div
            role="tabpanel"
            hidden={value !== index}
            id={`simple-tabpanel-${index}`}
            aria-labelledby={`simple-tab-${index}`}
            {...other}
        >
            {value === index && (
                <Box sx={{ p: 3 }}>
                    <Typography>{children}</Typography>
                </Box>
            )}
        </div>
    );
}

function a11yProps(index: number) {
    return {
        id: `simple-tab-${index}`,
        'aria-controls': `simple-tabpanel-${index}`,
    };
}


function filterByHours(peers, from) {
    const now = new Date()
    const hours = now.getHours()
    const day = now.getDay()

    const last_peers = peers.filter(peer => new Date(peer.time_last_request).getTime() > new Date(now.setHours(hours-from)).getTime())
    const this_day_peers = last_peers.filter(peer => new Date(peer.time_last_request).getDay() === day).map(peer => new Date(peer.time_last_request).getHours())
    const another_day_peers = last_peers.filter(peer => new Date(peer.time_last_request).getDay() > day).map(peer => new Date(peer.time_last_request).getHours())


    const data = []

    if (from > hours) {
            const inicio = from - hours
            for (let i = 24 - inicio + 1; i < 24; i++) {
                const bucket = another_day_peers.filter(hour => hour === i)
                console.log("bucket ayer: ", i, bucket)
                data.push(bucket.length)
            }

            for (let i = 0; i <= hours; i++) {
                const bucket = this_day_peers.filter(hour => hour === i)
                console.log("bucket hoy: ", i, bucket)
                data.push(bucket.length)
            }
    }
    else {
        for (let i = from; i > 0; i--) {
            const bucket = last_peers.filter(peer => (new Date(peer.time_last_request).getTime() > new Date(now.setHours(hours-i)).getTime()) && (new Date(peer.time_last_request).getTime() < new Date(now.setHours(hours-i+1)).getTime()))
            data.push(bucket.length)
        }
    }
    console.log(data)
    return data
}

function filterByHourTrackerConnection(response, from) {
    const now = new Date()
    const hours = now.getHours()
    const day = now.getDay()

    const last_peers = response.data.info.filter(peer => new Date(peer.time_last_request).getTime() > new Date(now.setHours(hours - from)).getTime())
    const today_peers = last_peers.filter(peer => new Date(peer.time_last_request).getDay() === day)
    const another_day_peers = last_peers.filter(peer => new Date(peer.time_last_request).getDay() > day)

    const data = []

    if (from > hours) {
        const inicio = from - hours
        for (let i = 24 - inicio + 1; i < 24; i++) {
            const bucket = another_day_peers.filter(peer => new Date(peer.time_last_request).getHours() === i).map(peer => peer.torrent)
            const uniq = [...new Set(bucket)];
            data.push(uniq.length)
        }

        for (let i = 0; i <= hours; i++) {
            const bucket = today_peers.filter(peer => new Date(peer.time_last_request).getHours() === i).map(peer => peer.torrent)
            const uniq = [...new Set(bucket)];
            data.push(uniq.length)
        }

    } else {
        for (let i = from; i > 0; i--) {
            const peer_by_bucket = last_peers.filter(peer => (new Date(peer.time_last_request).getTime() > new Date(now.setHours(hours - i)).getTime()) && (new Date(peer.time_last_request).getTime() < new Date(now.setHours(hours - i + 1)).getTime()))
            const bucket = peer_by_bucket.map(peer => peer.torrent)
            const uniq = [...new Set(bucket)];
            data.push(uniq.length)
        }
    }
    return data
}

function filterByDay(peers, from) {
    const now = new Date()
    const date = now.getDate()
    const last_peers = peers.filter(peer => new Date(peer.time_last_request).getDate() >  date - from).map(peer => new Date(peer.time_last_request).getDate())

    const data = []
    for (let i = date-from+1; i <= date; i++) {
        const bucket = last_peers.filter(day => day === i)
        data.push(bucket.length)
    }
    return data
}

function filterByDayTrackerConnection(response, from) {
    const now = new Date()
    const date = now.getDate()
    const last_peers = response.data.info.filter(peer => new Date(peer.time_last_request).getDate() >  date - from)

    const data = []
    for (let i = date-from+1; i <= date; i++) {
        const peer_by_bucket = last_peers.filter(peer => new Date(peer.time_last_request).getDate() === i)
        const bucket = peer_by_bucket.map(peer => peer.torrent)
        const uniq = [...new Set(bucket)];
        data.push(uniq.length)
    }
    return data
}


function filterByHourPeersState(response, from, state) {
    let peers = response.data.info;
    if (state) {
        peers = response.data.info.filter(peer => peer.completed)
    }
    return filterByHours(peers, from)
}

function filterByDayPeersState(response, from, state){
    let peers = response.data.info;
    if (state) {
        peers = response.data.info.filter(peer => peer.completed)
    }
    return filterByDay(peers, from)
}


function getAxis(size) {
    const axis = []
    for (let i = size; i > 1; i--) { axis.push('Hace '+ i + ' horas') }
    axis.push('Hace 1 hora')
    return axis
}

function getDayAxis(size) {
    const axis = []
    for (let i = size; i > 1; i--) { axis.push('Hace '+ i + ' dias') }
    axis.push('Hace 1 dia')
    return axis
}

export default function App() {
    const [value, setValue] = React.useState(0);
    const [response, setResponse] = React.useState(MOCK_TORRENT);

    React.useEffect(() => {
        axios.get("http://127.0.0.1:8080/stats", {
            method: "get",
        }).then((response) => {
           console.log(response);
           setResponse(response);
        });
    }, []);

    const handleChange = (event: React.SyntheticEvent, newValue: number) => {
        setValue(newValue);
    };

    return (
        <Box sx={{ width: '100%' }}>
            <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
                <h1>Bit Torrent - 4Rustasticos</h1>
                <Tabs value={value} onChange={handleChange} aria-label="basic tabs example" textColor="secondary"
                      TabIndicatorProps={{
                          style: {
                              backgroundColor: "rgba(238,180,202,0.36)",
                              background: "deeppink", height: "10px", top: "35px"
                          }
                      }}>
                    <Tab label="Última hora" {...a11yProps(0)} />
                    <Tab label="Últimas 5 horas" {...a11yProps(1)} />
                    <Tab label="Último día" {...a11yProps(2)} />
                    <Tab label="Últimos 3 días" {...a11yProps(3)} />
                </Tabs>
            </Box>

            <TabPanel value={value} index={0}>
                <h2>Peers activos</h2>
                <BarGraph props={{xaxis: getAxis(1), data: filterByHourPeersState(response,1, false)}}/>

                <h2>Peers que completaron la descarga</h2>
                <BarGraph props={{xaxis: getAxis(1), data: filterByHourPeersState(response,1, true)}}/>

                <h2>Torrents descargados</h2>
                <BarGraph props={{xaxis: getAxis(1), data: filterByHourTrackerConnection(response, 1)}}/>
            </TabPanel>

            <TabPanel value={value} index={1}>
                <h2>Peers activos</h2>
                <BarGraph props={{xaxis: getAxis(5), data: filterByHourPeersState(response, 5, false)}}/>

                <h2>Peers que completaron la descarga</h2>
                <BarGraph props={{xaxis: getAxis(5), data: filterByHourPeersState(response, 5, true)}}/>

                <h2>Torrents descargados</h2>
                <BarGraph props={{xaxis: getAxis(5), data: filterByHourTrackerConnection(response, 5)}}/>
            </TabPanel>

            <TabPanel value={value} index={2}>
                <h2>Peers activos</h2>
                <BarGraph props={{xaxis: getAxis(24), data: filterByHourPeersState(response, 24, false)}}/>

                <h2>Peers que completaron la descarga</h2>
                <BarGraph props={{xaxis: getAxis(24), data: filterByHourPeersState(response, 24, true)}}/>

                <h2>Torrents descargados</h2>
                <BarGraph props={{xaxis: getAxis(24), data: filterByHourTrackerConnection(response,24)}}/>
            </TabPanel>

            <TabPanel value={value} index={3}>
                <h2>Peers activos</h2>
                <BarGraph props={{xaxis: getDayAxis(3), data: filterByDayPeersState(response,3, false)}}/>

                <h2>Peers que completaron la descarga</h2>
                <BarGraph props={{xaxis: getDayAxis(3), data: filterByDayPeersState(response,3, true)}}/>

                <h2>Torrents descargados</h2>
                <BarGraph props={{xaxis: getDayAxis(3), data: filterByDayTrackerConnection(response,3)}}/>
            </TabPanel>

        </Box>
    );
}
